use chrono::SecondsFormat;
use chrono::Utc;
use cis_profile::crypto::Signer;
use cis_profile::schema::Display;
use cis_profile::schema::KeyValue;
use cis_profile::schema::Profile;
use cis_profile::schema::PublisherAuthority;
use cis_profile::schema::StandardAttributeString;
use cis_profile::schema::StandardAttributeValues;
use failure::Error;
use juniper::GraphQLInputObject;
use std::collections::BTreeMap;

fn update_string(
    s: &Option<StringWithDisplay>,
    p: &mut StandardAttributeString,
    now: &str,
    store: &impl Signer,
) -> Result<(), Error> {
    if let Some(x) = s {
        let mut sign = false;
        if x.value != p.value {
            if let Some(value) = &x.value {
                p.value = Some(value.clone());
                sign = true;
            }
        }
        if x.display != p.metadata.display {
            if let Some(display) = &x.display {
                p.metadata.display = Some(display.clone());
                sign = true;
            }
        }
        if sign {
            p.metadata.last_modified = now.to_owned();
            p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            store.sign_attribute(p)?;
        }
    }
    Ok(())
}

fn update_key_values(
    s: &Option<KeyValuesWithDisplay>,
    p: &mut StandardAttributeValues,
    now: &str,
    store: &impl Signer,
) -> Result<(), Error> {
    if let Some(x) = s {
        let mut sign = false;
        if let Some(values) = &x.values {
            let values: BTreeMap<String, Option<String>> =
                values.iter().map(|e| (e.k.clone(), e.v.clone())).collect();
            let kv = Some(KeyValue(values));
            if kv != p.values {
                p.values = kv;
            }
            sign = true;
        }
        if x.display != p.metadata.display {
            if let Some(display) = &x.display {
                p.metadata.display = Some(display.clone());
                sign = true;
            }
        }
        if sign {
            p.metadata.last_modified = now.to_owned();
            p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            store.sign_attribute(p)?;
        }
    }
    Ok(())
}

#[derive(GraphQLInputObject, Default)]
pub struct BoolWithDisplay {
    display: Option<Display>,
    value: Option<bool>,
}

#[derive(GraphQLInputObject, Default)]
pub struct StringWithDisplay {
    display: Option<Display>,
    value: Option<String>,
}

#[derive(GraphQLInputObject, Default)]
pub struct KeyValueInput {
    k: String,
    v: Option<String>,
}

#[derive(GraphQLInputObject, Default)]
pub struct KeyValuesWithDisplay {
    display: Option<Display>,
    values: Option<Vec<KeyValueInput>>,
}

#[derive(GraphQLInputObject, Default)]
pub struct InputProfile {
    pub active: Option<BoolWithDisplay>,
    pub alternative_name: Option<StringWithDisplay>,
    pub created: Option<StringWithDisplay>,
    pub description: Option<StringWithDisplay>,
    pub first_name: Option<StringWithDisplay>,
    pub fun_title: Option<StringWithDisplay>,
    //pub identities: IdentitiesAttributesValuesArray,
    pub languages: Option<KeyValuesWithDisplay>,
    pub last_modified: Option<StringWithDisplay>,
    pub last_name: Option<StringWithDisplay>,
    pub location: Option<StringWithDisplay>,
    pub login_method: Option<StringWithDisplay>,
    pub pgp_public_keys: Option<KeyValuesWithDisplay>,
    pub phone_numbers: Option<KeyValuesWithDisplay>,
    pub picture: Option<StringWithDisplay>,
    pub primary_email: Option<StringWithDisplay>,
    pub primary_username: Option<StringWithDisplay>,
    pub pronouns: Option<StringWithDisplay>,
    pub ssh_public_keys: Option<KeyValuesWithDisplay>,
    //pub staff_information: StaffInformationValuesArray,
    pub tags: Option<KeyValuesWithDisplay>,
    pub timezone: Option<StringWithDisplay>,
    pub uris: Option<KeyValuesWithDisplay>,
    pub user_id: Option<StringWithDisplay>,
    pub usernames: Option<KeyValuesWithDisplay>,
}

impl InputProfile {
    pub fn update_profile(&self, p: &mut Profile, secret_store: &impl Signer) -> Result<(), Error> {
        let now = &Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        update_string(
            &self.alternative_name,
            &mut p.alternative_name,
            now,
            secret_store,
        )?;
        update_string(&self.created, &mut p.created, now, secret_store)?;
        update_string(&self.description, &mut p.description, now, secret_store)?;
        update_string(&self.first_name, &mut p.first_name, now, secret_store)?;
        update_string(&self.fun_title, &mut p.fun_title, now, secret_store)?;
        update_string(&self.last_modified, &mut p.last_modified, now, secret_store)?;
        update_string(&self.last_name, &mut p.last_name, now, secret_store)?;
        update_string(&self.location, &mut p.location, now, secret_store)?;
        update_string(&self.login_method, &mut p.login_method, now, secret_store)?;
        update_string(&self.picture, &mut p.picture, now, secret_store)?;
        update_string(&self.primary_email, &mut p.primary_email, now, secret_store)?;
        update_string(
            &self.primary_username,
            &mut p.primary_username,
            now,
            secret_store,
        )?;
        update_string(&self.pronouns, &mut p.pronouns, now, secret_store)?;
        update_string(&self.timezone, &mut p.timezone, now, secret_store)?;
        update_string(&self.user_id, &mut p.user_id, now, secret_store)?;

        update_key_values(&self.languages, &mut p.languages, now, secret_store)?;
        update_key_values(&self.phone_numbers, &mut p.phone_numbers, now, secret_store)?;
        update_key_values(&self.tags, &mut p.tags, now, secret_store)?;
        update_key_values(&self.usernames, &mut p.usernames, now, secret_store)?;
        update_key_values(&self.uris, &mut p.uris, now, secret_store)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cis_profile::crypto::SecretStore;
    use cis_profile::schema::Profile;

    fn get_fake_secret_store() -> SecretStore {
        let v = vec![(
            String::from("mozilliansorg"),
            String::from(include_str!("../../tests/data/fake_key.json")),
        )];
        SecretStore::default()
            .with_sign_keys_from_inline_iter(v)
            .unwrap()
    }

    #[test]
    fn test_simple_update() -> Result<(), Error> {
        let secret_store = get_fake_secret_store();
        let mut p = Profile::default();
        let mut update = InputProfile::default();
        update.fun_title = Some(StringWithDisplay {
            value: Some(String::from("Pope")),
            display: None,
        });
        assert!(p.fun_title.value.is_none());
        update.update_profile(&mut p, &secret_store)?;
        assert_eq!(p.fun_title.value, update.fun_title.unwrap().value);
        Ok(())
    }
}
