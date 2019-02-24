use biscuit::jws;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use reqwest::Client;

use serde_json::Value;

use crate::remote_store::RemoteGet;
use crate::settings::ClientConfig;

pub struct BaererBaerer {
    pub baerer_token_str: String,
    pub exp: DateTime<Utc>,
    pub config: ClientConfig,
}

impl RemoteGet for BaererBaerer {
    fn get(&mut self) -> Result<(), String> {
        self.baerer_token_str = get_raw_access_token(&self.config)?;
        self.exp = get_expiration(&self.baerer_token_str)?;
        Ok(())
    }
    fn expiry(&self) -> DateTime<Utc> {
        self.exp
    }
}

impl BaererBaerer {
    pub fn new(config: ClientConfig) -> Self {
        BaererBaerer {
            baerer_token_str: String::default(),
            exp: Utc.timestamp(0, 0),
            config,
        }
    }
}

fn get_expiration(token: &str) -> Result<DateTime<Utc>, String> {
    let c: jws::Compact<biscuit::ClaimsSet<Value>, biscuit::Empty> =
        jws::Compact::new_encoded(&token);
    let payload = c
        .unverified_payload()
        .map_err(|e| format!("unable to get payload from token: {}", e))?;
    let exp = payload
        .registered
        .expiry
        .ok_or_else(|| String::from("no expiration set in token"))?;
    Ok(*exp)
}

pub fn get_raw_access_token(client_config: &ClientConfig) -> Result<String, String> {
    let payload = json!(
        {
            "client_id": client_config.client_id,
            "client_secret": client_config.client_secret,
            "audience": client_config.audience,
            "grant_type": "client_credentials",
            "scopes": "read:fullprofile display:all"
        }
    );
    let client = Client::new();
    let mut res: reqwest::Response = client
        .post("https://auth.mozilla.auth0.com/oauth/token")
        .json(&payload)
        .send()
        .map_err(|e| format!("can't get token: {}", e))?;
    let j: serde_json::Value = res
        .json()
        .map_err(|e| format!("can't parse token: {}", e))?;
    j["access_token"]
        .as_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| String::from("no token :/"))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn test_get_access_token() {
        if let Ok(s) = Settings::new() {
            let r = get_raw_access_token(&s.cis.client_config);
            assert!(r.is_ok());
        }
    }
}
