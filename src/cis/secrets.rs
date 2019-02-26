use crate::settings::Settings;
use cis_profile::crypto::SecretStore;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub fn get_store_from_settings(settings: &Settings) -> Result<SecretStore, String> {
    match settings.cis.keys.source.as_str() {
        "file" => get_store_from_files(settings),
        "ssm" => get_store_from_ssm(settings),
        _ => Err(String::from("invalid key source: use 'file' or 'ssm'")),
    }
}

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

pub fn get_store_from_files(settings: &Settings) -> Result<SecretStore, String> {
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
    .filter_map(|(k, v)| v.map(|v| (k, v)))
    .map(|(k, v)| read_file(&v).map(|content| (k, content)))
    .collect::<Result<Vec<(String, String)>, String>>()?;
    SecretStore::from_inline_iter(keys)
}

fn read_file(file_name: &str) -> Result<String, String> {
    let file =
        File::open(file_name).map_err(|e| format!("unable to open file '{}': {}", file_name, e))?;
    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();
    buf_reader
        .read_to_string(&mut content)
        .map_err(|e| format!("unable to read file '{}': {}", file_name, e))?;
    Ok(content)
}
