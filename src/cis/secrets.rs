use crate::settings::Settings;
use cis_profile::crypto::SecretStore;

pub fn get_store_from_ssm(settings: &Settings) -> Result<SecretStore, String> {
    let keys = vec![
        (
            String::from("mozilliansorg"),
            Some(settings.cis.keys.mozilliansorg_key.clone()),
        ),
        (String::from("hris"), settings.cis.keys.hris_key.clone()),
        (String::from("ldap"), settings.cis.keys.ldap_key.clone()),
        (String::from("cis"), settings.cis.keys.cis_key.clone()),
        (
            String::from("access_provider"),
            settings.cis.keys.access_provider_key.clone(),
        ),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|v| (k, v)));
    SecretStore::from_ssm_iter(keys)
}
