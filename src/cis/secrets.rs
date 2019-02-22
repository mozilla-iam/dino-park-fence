use cis_profile::crypto::SecretStore;
use std::env;

pub fn get_store_from_ssm_via_env() -> Result<SecretStore, String> {
    if let (
        Ok(mozillians_key_ssm_name),
        Ok(hris_key_ssm_name),
        Ok(ldap_key_ssm_name),
        Ok(cis_key_ssm_name),
        Ok(access_provider_key_ssm_name),
    ) = (
        env::var("CIS_SSM_MOZILLIANSORG_KEY"),
        env::var("CIS_SSM_HRIS_KEY"),
        env::var("CIS_SSM_LDAP_KEY"),
        env::var("CIS_SSM_CIS_KEY"),
        env::var("CIS_SSM_ACCESS_PROVIDER_KEY"),
    ) {
        SecretStore::from_ssm_iter(vec![
            (String::from("mozilliansorg"), mozillians_key_ssm_name),
            (String::from("hris"), hris_key_ssm_name),
            (String::from("ldap"), ldap_key_ssm_name),
            (String::from("cis"), cis_key_ssm_name),
            (
                String::from("access_provider"),
                access_provider_key_ssm_name,
            ),
        ])
    } else {
        Err(String::from("missing CIS_SSM_XXX environment variables"))
    }
}
