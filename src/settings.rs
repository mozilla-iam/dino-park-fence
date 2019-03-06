use cis_client::settings::CisSettings;
use config::{Config, ConfigError, Environment, File};
use std::env;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub cis: CisSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let file = env::var("DPF_SETTINGS").unwrap_or_else(|_| String::from(".settings"));
        let mut s = Config::new();
        s.merge(File::with_name(&file))?;
        s.merge(Environment::new().separator("__"))?;
        s.try_into()
    }
}
