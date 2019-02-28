use chrono::SecondsFormat;
use chrono::Utc;
use cis_profile::crypto::sign_attribute;
use cis_profile::crypto::SecretStore;
use cis_profile::schema::Display;
use cis_profile::schema::Profile;
use cis_profile::schema::PublisherAuthority;
use cis_profile::schema::StandardAttributeString;
use juniper::GraphQLInputObject;

fn update_string(
    s: &Option<StringWithDisplay>,
    p: &mut StandardAttributeString,
    now: &str,
    store: &SecretStore,
) -> Result<(), String> {
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
            sign_attribute(p, store)?;
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
    key: String,
    value: Option<String>,
}

#[derive(GraphQLInputObject, Default)]
pub struct KeyValuesWithDisplay {
    display: Option<Display>,
    value: Option<Vec<KeyValueInput>>,
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
    pub uuid: Option<StringWithDisplay>,
}

impl InputProfile {
    #[allow(clippy::cyclomatic_complexity)]
    pub fn update_profile(
        &self,
        p: &mut Profile,
        secret_store: &SecretStore,
    ) -> Result<(), String> {
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
        update_string(&self.uuid, &mut p.uuid, now, secret_store)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cis_profile::schema::Profile;

    fn get_fake_secret_store() -> SecretStore {
        let v = vec![(
            String::from("mozilliansorg"),
            String::from(include_str!("../../tests/data/fake_key.json")),
        )];
        SecretStore::from_inline_iter(v).unwrap()
    }

    #[test]
    fn test_simple_update() -> Result<(), String> {
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
