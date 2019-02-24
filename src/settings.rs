use config::{Config, ConfigError, Environment, File};

#[derive(Clone, Debug, Deserialize)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub audience: String,
}

#[derive(Debug, Deserialize)]
pub struct Cis {
    pub person_api_user_endpoint: String,
    pub change_api_user_endpoint: String,
    pub client_config: ClientConfig,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub cis: Cis,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(".settings"))?;
        s.merge(Environment::new().separator("__"))?;
        s.try_into()
    }
}
