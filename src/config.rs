use crate::auth::BaererBaerer;
use crate::remote_store::RemoteStore;
use crate::cis_profile::crypto::SecretStore;

use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub bearer_store: RemoteStore<BaererBaerer>,
    pub secret_store: Arc<SecretStore>,
}
