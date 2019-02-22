use cis_profile::crypto::SecretStore;
use cis_profile::schema::Profile;
use cis_profile::utils::sign_full_profile;
use percent_encoding::utf8_percent_encode;
use percent_encoding::USERINFO_ENCODE_SET;
use reqwest::Client;
use serde_json::Value;
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

pub fn get_user(
    bearer_token: &str,
    id: &str,
    by: &GetBy,
    display: Option<&str>,
) -> Result<Profile, String> {
    let safe_id = utf8_percent_encode(id, USERINFO_ENCODE_SET).to_string();
    let base = Url::parse("https://person.api.dev.sso.allizom.org/v2/user/")
        .map_err(|e| format!("{}", e))?;
    let url = base
        .join(by.as_str())
        .and_then(|u| u.join(&safe_id))
        .map(|mut u| {
            if let Some(dl) = display {
                u.set_query(Some(&format!("filterDisplay={}", dl)))
            }
            u
        })
        .map_err(|e| format!("{}", e))?;
    let client = Client::new().get(url.as_str()).bearer_auth(bearer_token);
    let mut res: reqwest::Response = client.send().map_err(|e| format!("{}", e))?;
    if res.status().is_success() {
        res.json()
            .map_err(|e| format!("Invalid JSON from user endpoint: {}", e))
    } else {
        Err(format!("person API returned: {}", res.status()))
    }
}

pub fn update_user(
    bearer_token: &str,
    sign: bool,
    mut profile: Profile,
    secret_store: &SecretStore,
) -> Result<Value, String> {
    if sign {
        sign_full_profile(&mut profile, secret_store)?;
    }
    let client = Client::new()
        .post("https://change.api.dev.sso.allizom.org/v2/user")
        .json(&profile)
        .bearer_auth(bearer_token);
    let mut res: reqwest::Response = client.send().map_err(|e| format!("change.api: {}", e))?;
    res.json()
        .map_err(|e| format!("change.api → json: {} ({:?})", e, res))
}
