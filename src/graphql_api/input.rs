use chrono::SecondsFormat;
use chrono::Utc;
use cis_profile::crypto::sign_attribute;
use cis_profile::crypto::SecretStore;
use cis_profile::schema::Display;
use cis_profile::schema::Profile;
use cis_profile::schema::PublisherAuthority;
use juniper::GraphQLInputObject;

macro_rules! update_simple_field {
    ($field:ident, $p:ident, $s:ident, $store:ident, $now:ident) => {
        if let Some(x) = &$s.$field {
            let mut sign = false;
            if x.value != $p.$field.value {
                if let Some(value) = &x.value {
                    $p.$field.value = Some(value.clone());
                    sign = true;
                }
            }
            if x.display != $p.$field.metadata.display {
                if let Some(display) = &x.display {
                    $p.$field.metadata.display = Some(display.clone());
                    sign = true;
                }
            }
            if sign {
                $p.$field.metadata.last_modified = $now.clone();
                $p.$field.signature.publisher.name = PublisherAuthority::Mozilliansorg;
                sign_attribute(&mut $p.$field, $store)?;
            }
        }
    };
}

#[derive(GraphQLInputObject)]
pub struct BoolWithDisplay {
    display: Option<Display>,
    value: Option<bool>,
}

#[derive(GraphQLInputObject)]
pub struct StringWithDisplay {
    display: Option<Display>,
    value: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct KeyValueInput {
    key: String,
    value: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct KeyValuesWithDisplay {
    display: Option<Display>,
    value: Option<Vec<KeyValueInput>>,
}

#[derive(GraphQLInputObject)]
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
        let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        update_simple_field!(alternative_name, p, self, secret_store, now);
        update_simple_field!(created, p, self, secret_store, now);
        update_simple_field!(description, p, self, secret_store, now);
        update_simple_field!(first_name, p, self, secret_store, now);
        update_simple_field!(fun_title, p, self, secret_store, now);
        update_simple_field!(last_modified, p, self, secret_store, now);
        update_simple_field!(last_name, p, self, secret_store, now);
        update_simple_field!(location, p, self, secret_store, now);
        update_simple_field!(login_method, p, self, secret_store, now);
        update_simple_field!(picture, p, self, secret_store, now);
        update_simple_field!(primary_email, p, self, secret_store, now);
        update_simple_field!(primary_username, p, self, secret_store, now);
        update_simple_field!(pronouns, p, self, secret_store, now);
        update_simple_field!(timezone, p, self, secret_store, now);
        update_simple_field!(user_id, p, self, secret_store, now);
        update_simple_field!(uuid, p, self, secret_store, now);
        Ok(())
    }
}
