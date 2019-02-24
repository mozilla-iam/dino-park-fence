use crate::cis::auth::BaererBaerer;
use crate::remote_store::RemoteStore;
use crate::settings::Settings;

#[derive(Clone)]
pub struct CisClient {
    pub bearer_store: RemoteStore<BaererBaerer>,
    pub person_api_user_endpoint: String,
    pub change_api_user_endpoint: String,
}

impl CisClient {
    pub fn from_settings(settings: &Settings) -> Result<Self, String> {
        let bearer_store = RemoteStore::new(BaererBaerer::new(settings.cis.client_config.clone()));
        Ok(CisClient {
            bearer_store,
            person_api_user_endpoint: settings.cis.person_api_user_endpoint.clone(),
            change_api_user_endpoint: settings.cis.change_api_user_endpoint.clone(),
        })
    }
}
