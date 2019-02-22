use biscuit::jws;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use reqwest::Client;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use serde_json::Value;

use crate::remote_store::RemoteGet;

#[derive(Default)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub audience: String,
}

pub struct BaererBaerer {
    pub baerer_token_str: String,
    pub exp: DateTime<Utc>,
    pub config: ClientConfig,
}

impl Default for BaererBaerer {
    fn default() -> Self {
        BaererBaerer {
            baerer_token_str: String::default(),
            exp: Utc.timestamp(0, 0),
            config: ClientConfig::default(),
        }
    }
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

fn load_json(path: impl Into<PathBuf>) -> Result<Value, String> {
    let mut s = String::new();
    File::open(path.into())
        .map_err(|e| format!("{}", e))?
        .read_to_string(&mut s)
        .map_err(|e| format!("{}", e))?;
    serde_json::from_str(&s).map_err(|e| format!("{}", e))
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

pub fn read_client_config(config_file: &str) -> Result<ClientConfig, String> {
    let config = load_json(config_file)?;
    let client_id = if let Some(client_id) = config["client_id"].as_str() {
        String::from(client_id)
    } else {
        return Err(String::from("missing client_id in config"));
    };
    let client_secret = if let Some(client_secret) = config["client_secret"].as_str() {
        String::from(client_secret)
    } else {
        return Err(String::from("missing client_secret in config"));
    };
    let audience = if let Some(audience) = config["audience"].as_str() {
        String::from(audience)
    } else {
        return Err(String::from("missing audience in config"));
    };
    Ok(ClientConfig {
        client_id,
        client_secret,
        audience,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_access_token() {
        if let Ok(cfg) = read_client_config(".person-api.json") {
            let r = get_raw_access_token(&cfg);
            assert!(r.is_ok());
        }
    }
}
