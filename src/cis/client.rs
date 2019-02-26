use crate::cis::auth::BaererBaerer;
use crate::cis::secrets::get_store_from_settings;
use crate::remote_store::RemoteStore;
use crate::settings::Settings;
use cis_profile::crypto::SecretStore;
use cis_profile::schema::Profile;
use percent_encoding::utf8_percent_encode;
use percent_encoding::USERINFO_ENCODE_SET;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use url::Url;

pub enum GetBy {
    UserId,
    PrimaryUsername,
}

impl GetBy {
    pub fn as_str(self: &GetBy) -> &'static str {
        match self {
            GetBy::UserId => "user_id/",
            GetBy::PrimaryUsername => "primary_username/",
        }
    }
}

#[derive(Clone)]
pub struct CisClient {
    pub bearer_store: RemoteStore<BaererBaerer>,
    pub person_api_user_endpoint: String,
    pub change_api_user_endpoint: String,
    pub secret_store: Arc<SecretStore>,
}

impl CisClient {
    pub fn from_settings(settings: &Settings) -> Result<Self, String> {
        let bearer_store = RemoteStore::new(BaererBaerer::new(settings.cis.client_config.clone()));
        let secret_store = get_store_from_settings(settings)?;
        Ok(CisClient {
            bearer_store,
            person_api_user_endpoint: settings.cis.person_api_user_endpoint.clone(),
            change_api_user_endpoint: settings.cis.change_api_user_endpoint.clone(),
            secret_store: Arc::new(secret_store),
        })
    }
}

impl CisClient {
    pub fn get_user_by(
        &self,
        id: &str,
        by: &GetBy,
        filter: Option<&str>,
    ) -> Result<Profile, String> {
        let safe_id = utf8_percent_encode(id, USERINFO_ENCODE_SET).to_string();
        let base = Url::parse("https://person.api.dev.sso.allizom.org/v2/user/")
            .map_err(|e| format!("{}", e))?;
        let url = base
            .join(by.as_str())
            .and_then(|u| u.join(&safe_id))
            .map(|mut u| {
                if let Some(df) = filter {
                    u.set_query(Some(&format!("filterDisplay={}", df.to_string())))
                }
                u
            })
            .map_err(|e| format!("{}", e))?;
        let b = self
            .bearer_store
            .get()
            .map_err(|e| format!("{}: {}", "unable to get token", e))?;
        let b1 = b
            .read()
            .map_err(|e| format!("{}: {}", "unable to read token", e))?;
        let token = &*b1.baerer_token_str;
        let client = Client::new().get(url.as_str()).bearer_auth(token);
        let mut res: reqwest::Response = client.send().map_err(|e| format!("{}", e))?;
        if res.status().is_success() {
            res.json()
                .map_err(|e| format!("Invalid JSON from user endpoint: {}", e))
        } else {
            Err(format!("person API returned: {}", res.status()))
        }
    }

    pub fn update_user(&self, profile: Profile) -> Result<Value, String> {
        let b = self
            .bearer_store
            .get()
            .map_err(|e| format!("{}: {}", "unable to get token", e))?;
        let b1 = b
            .read()
            .map_err(|e| format!("{}: {}", "unable to read token", e))?;
        let token = &*b1.baerer_token_str;
        let client = Client::new()
            .post("https://change.api.dev.sso.allizom.org/v2/user")
            .json(&profile)
            .bearer_auth(token);
        let mut res: reqwest::Response = client.send().map_err(|e| format!("change.api: {}", e))?;
        res.json()
            .map_err(|e| format!("change.api → json: {} ({:?})", e, res))
    }

    pub fn get_secret_store(&self) -> &SecretStore {
        &self.secret_store
    }
}
