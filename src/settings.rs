use cis_client::settings::CisSettings;
use config::{Config, ConfigError, Environment, File};
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Orgchart {
    pub related_endpoint: String,
    pub full_endpoint: String,
    pub trace_endpoint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Search {
    pub simple_endpoint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fossil {
    pub upload_endpoint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Lookout {
    pub internal_update_endpoint: String,
    pub internal_update_enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DinoParkServices {
    pub orgchart: Orgchart,
    pub search: Search,
    pub fossil: Fossil,
    pub lookout: Lookout,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub cis: CisSettings,
    pub dino_park: DinoParkServices,
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
