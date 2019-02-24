use crate::cis::client::CisClient;
use cis_profile::crypto::SecretStore;

use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub cis_client: CisClient,
    pub secret_store: Arc<SecretStore>,
}
