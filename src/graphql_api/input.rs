use crate::graphql_api::avatar::change_picture_display;
use crate::graphql_api::avatar::upload_picture;
use crate::settings::Fossil;
use chrono::SecondsFormat;
use chrono::Utc;
use cis_profile::crypto::Signer;
use cis_profile::schema::Display;
use cis_profile::schema::IdentitiesAttributesValuesArray;
use cis_profile::schema::KeyValue;
use cis_profile::schema::Profile;
use cis_profile::schema::PublisherAuthority;
use cis_profile::schema::StandardAttributeString;
use cis_profile::schema::StandardAttributeValues;
use failure::Error;
use juniper::GraphQLInputObject;
use std::collections::BTreeMap;

fn create_usernames_key(typ: &str) -> String {
    format!("HACK#{}", typ)
}

fn update_picture(
    s: &Option<StringWithDisplay>,
    p: &mut StandardAttributeString,
    uuid: &StandardAttributeString,
    now: &str,
    store: &impl Signer,
    fossil_settings: &Fossil,
) -> Result<(), Error> {
    if let Some(new_picture) = s {
        let mut changed = false;
        if new_picture.display != p.metadata.display {
            if let Some(display) = &new_picture.display {
                // if display changed but field is null change it to empty string
                if p.value.is_none() {
                    p.value = Some(String::default());
                }
                p.metadata.display = Some(display.clone());
                changed = true;
            }
        }
        if new_picture.value != p.value && new_picture.value != Some(String::default()) {
            if let Some(display) = &p.metadata.display {
                if let Some(value) = &new_picture.value {
                    let uuid = uuid
                        .value
                        .as_ref()
                        .ok_or_else(|| failure::err_msg("no uuid in profile"))?;
                    let url = upload_picture(
                        &value,
                        uuid,
                        &display,
                        p.value.as_ref().map(String::as_str),
                        &fossil_settings.upload_endpoint,
                    )?;
                    p.value = Some(url);
                    changed = true;
                }
            }
        } else if changed && p.value != Some(String::default()) {
            // if only the display level changed we have to send a display update to fossil
            if let Some(display) = &p.metadata.display {
                let uuid = uuid
                    .value
                    .as_ref()
                    .ok_or_else(|| failure::err_msg("no uuid in profile"))?;
                let url = change_picture_display(
                    uuid,
                    &display,
                    p.value.as_ref().map(String::as_str),
                    &fossil_settings.upload_endpoint,
                )?;
                p.value = Some(url);
                changed = true;
            }
        } else if new_picture.value != p.value && new_picture.value == Some(String::default()) {
            // TODO: delete picture
            p.value = new_picture.value.clone();
            changed = true;
        }

        if changed {
            p.metadata.last_modified = now.to_owned();
            p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            store.sign_attribute(p)?;
        }
    }
    Ok(())
}

fn update_bugzilla_identity(
    bugzilla: &IdentityWithDisplay,
    p: &mut IdentitiesAttributesValuesArray,
    u: &mut StandardAttributeValues,
    now: &str,
    store: &impl Signer,
) -> Result<(), Error> {
    let mut sign_bugzilla = false;
    let mut sign_usernames = false;
    if bugzilla.remove.unwrap_or_default() {
        p.bugzilla_mozilla_org_id.metadata.display = Some(Display::Staff);
        p.bugzilla_mozilla_org_primary_email.metadata.display = Some(Display::Staff);

        if let Some(KeyValue(usernames)) = &mut u.values {
            if usernames.remove(&create_usernames_key("BMOMAIL")).is_some() {
                sign_usernames = true;
            }
            if usernames.remove(&create_usernames_key("BMONICK")).is_some() {
                sign_usernames = true;
            }
        }

        p.bugzilla_mozilla_org_id.value = Some(String::default());
        p.bugzilla_mozilla_org_primary_email.value = Some(String::default());
        sign_bugzilla = true;
    } else if bugzilla.display != p.bugzilla_mozilla_org_id.metadata.display
        || bugzilla.display != p.bugzilla_mozilla_org_primary_email.metadata.display
    {
        if let Some(display) = &bugzilla.display {
            if p.bugzilla_mozilla_org_id.value.is_none() {
                p.bugzilla_mozilla_org_id.value = Some(String::default())
            }
            if p.bugzilla_mozilla_org_primary_email.value.is_none() {
                p.bugzilla_mozilla_org_primary_email.value = Some(String::default())
            }

            p.bugzilla_mozilla_org_id.metadata.display = Some(display.clone());
            p.bugzilla_mozilla_org_primary_email.metadata.display = Some(display.clone());
            sign_bugzilla = true;
        }
    }

    if sign_bugzilla {
        p.bugzilla_mozilla_org_id.metadata.last_modified = now.to_owned();
        p.bugzilla_mozilla_org_primary_email.metadata.last_modified = now.to_owned();
        p.bugzilla_mozilla_org_id.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        p.bugzilla_mozilla_org_primary_email
            .signature
            .publisher
            .name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(&mut p.bugzilla_mozilla_org_id)?;
        store.sign_attribute(&mut p.bugzilla_mozilla_org_primary_email)?;
    }

    if sign_usernames {
        u.metadata.last_modified = now.to_owned();
        u.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(u)?;
    }

    Ok(())
}

fn update_github_identity(
    github: &IdentityWithDisplay,
    p: &mut IdentitiesAttributesValuesArray,
    u: &mut StandardAttributeValues,
    now: &str,
    store: &impl Signer,
) -> Result<(), Error> {
    let mut sign_github = false;
    let mut sign_usernames = false;
    if github.remove.unwrap_or_default() {
        p.github_id_v3.metadata.display = Some(Display::Staff);
        p.github_id_v4.metadata.display = Some(Display::Staff);
        p.github_primary_email.metadata.display = Some(Display::Staff);

        if let Some(KeyValue(usernames)) = &mut u.values {
            if usernames.remove(&create_usernames_key("GITHUB")).is_some() {
                sign_usernames = true;
            }
        }

        p.github_id_v3.value = Some(String::default());
        p.github_id_v4.value = Some(String::default());
        p.github_primary_email.value = Some(String::default());
        sign_github = true;
    } else if github.display != p.github_id_v3.metadata.display
        || github.display != p.github_id_v4.metadata.display
        || github.display != p.github_primary_email.metadata.display
    {
        if let Some(display) = &github.display {
            if p.github_id_v3.value.is_none() {
                p.github_id_v3.value = Some(String::default())
            }
            if p.github_id_v4.value.is_none() {
                p.github_id_v4.value = Some(String::default())
            }
            if p.github_primary_email.value.is_none() {
                p.github_primary_email.value = Some(String::default())
            }

            p.github_id_v3.metadata.display = Some(display.clone());
            p.github_id_v4.metadata.display = Some(display.clone());
            p.github_primary_email.metadata.display = Some(display.clone());
            sign_github = true;
        }
    }

    if sign_github {
        p.github_id_v3.metadata.last_modified = now.to_owned();
        p.github_id_v4.metadata.last_modified = now.to_owned();
        p.github_primary_email.metadata.last_modified = now.to_owned();
        p.github_id_v3.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        p.github_id_v4.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        p.github_primary_email.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(&mut p.github_id_v3)?;
        store.sign_attribute(&mut p.github_id_v4)?;
        store.sign_attribute(&mut p.github_primary_email)?;
    }

    if sign_usernames {
        u.metadata.last_modified = now.to_owned();
        u.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(u)?;
    }

    Ok(())
}

fn update_identities(
    i: &Option<IdentitiesWithDisplay>,
    p: &mut IdentitiesAttributesValuesArray,
    u: &mut StandardAttributeValues,
    now: &str,
    store: &impl Signer,
) -> Result<(), Error> {
    if let Some(identities) = i {
        if let Some(github) = &identities.github {
            update_github_identity(github, p, u, now, store)?;
        }
        if let Some(bugzilla) = &identities.bugzilla {
            update_bugzilla_identity(bugzilla, p, u, now, store)?;
        }
    }

    Ok(())
}

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
                // if display changed but field is null change it to empty string
                if p.value.is_none() {
                    p.value = Some(String::default());
                }
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
    filter_empty_values: bool,
) -> Result<(), Error> {
    if let Some(x) = s {
        let mut sign = false;
        if let Some(values) = &x.values {
            let values: BTreeMap<String, Option<String>> = if filter_empty_values {
                values
                    .iter()
                    .filter_map(|e| {
                        if e.v.as_ref().map(|s| !s.is_empty()).unwrap_or_default() {
                            None
                        } else {
                            Some((e.k.clone(), e.v.clone()))
                        }
                    })
                    .collect()
            } else {
                values.iter().map(|e| (e.k.clone(), e.v.clone())).collect()
            };
            let kv = Some(KeyValue(values));
            if kv != p.values {
                p.values = kv;
            }
            sign = true;
        }
        if x.display != p.metadata.display {
            if let Some(display) = &x.display {
                // if display changed but field is null change it to empty dict
                if p.values.is_none() {
                    p.values = Some(KeyValue(BTreeMap::default()));
                }
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
    pub display: Option<Display>,
    pub value: Option<bool>,
}

#[derive(GraphQLInputObject, Default)]
pub struct StringWithDisplay {
    pub display: Option<Display>,
    pub value: Option<String>,
}

#[derive(GraphQLInputObject, Default)]
pub struct KeyValueInput {
    pub k: String,
    pub v: Option<String>,
}

#[derive(GraphQLInputObject, Default)]
pub struct KeyValuesWithDisplay {
    pub display: Option<Display>,
    pub values: Option<Vec<KeyValueInput>>,
}

#[derive(GraphQLInputObject, Default)]
pub struct IdentityWithDisplay {
    pub remove: Option<bool>,
    pub display: Option<Display>,
}

#[derive(GraphQLInputObject, Default)]
pub struct IdentitiesWithDisplay {
    pub github: Option<IdentityWithDisplay>,
    pub bugzilla: Option<IdentityWithDisplay>,
}

#[derive(GraphQLInputObject, Default)]
pub struct InputProfile {
    pub active: Option<BoolWithDisplay>,
    pub alternative_name: Option<StringWithDisplay>,
    pub created: Option<StringWithDisplay>,
    pub description: Option<StringWithDisplay>,
    pub first_name: Option<StringWithDisplay>,
    pub fun_title: Option<StringWithDisplay>,
    pub identities: Option<IdentitiesWithDisplay>,
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
    pub tags: Option<KeyValuesWithDisplay>,
    pub timezone: Option<StringWithDisplay>,
    pub uris: Option<KeyValuesWithDisplay>,
    pub user_id: Option<StringWithDisplay>,
    pub usernames: Option<KeyValuesWithDisplay>,
}

impl InputProfile {
    pub fn update_profile(
        &self,
        p: &mut Profile,
        secret_store: &impl Signer,
        fossil_settings: &Fossil,
    ) -> Result<(), Error> {
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
        update_picture(
            &self.picture,
            &mut p.picture,
            &p.uuid,
            now,
            secret_store,
            &fossil_settings,
        )?;
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

        update_key_values(&self.languages, &mut p.languages, now, secret_store, false)?;
        update_key_values(
            &self.phone_numbers,
            &mut p.phone_numbers,
            now,
            secret_store,
            true,
        )?;
        update_key_values(&self.tags, &mut p.tags, now, secret_store, false)?;
        update_key_values(&self.usernames, &mut p.usernames, now, secret_store, true)?;
        update_key_values(&self.uris, &mut p.uris, now, secret_store, true)?;
        update_key_values(
            &self.pgp_public_keys,
            &mut p.pgp_public_keys,
            now,
            secret_store,
            true,
        )?;
        update_key_values(
            &self.ssh_public_keys,
            &mut p.ssh_public_keys,
            now,
            secret_store,
            true,
        )?;
        update_identities(
            &self.identities,
            &mut p.identities,
            &mut p.usernames,
            now,
            secret_store,
        )?;
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
        let fossil_settings = Fossil {
            upload_endpoint: String::default(),
        };
        let mut p = Profile::default();
        let mut update = InputProfile::default();
        update.fun_title = Some(StringWithDisplay {
            value: Some(String::from("Pope")),
            display: None,
        });
        assert_eq!(p.fun_title.value, None);
        update.update_profile(&mut p, &secret_store, &fossil_settings)?;
        assert_eq!(p.fun_title.value, update.fun_title.unwrap().value);
        Ok(())
    }

    #[test]
    fn test_update_display_only_with_null_value_string() -> Result<(), Error> {
        let secret_store = get_fake_secret_store();
        let fossil_settings = Fossil {
            upload_endpoint: String::default(),
        };
        let mut p = Profile::default();
        let mut update = InputProfile::default();
        update.fun_title = Some(StringWithDisplay {
            value: None,
            display: Some(Display::Vouched),
        });
        assert_eq!(p.pronouns.value, None);
        assert_eq!(p.fun_title.value, None);
        assert_ne!(p.fun_title.metadata.display, Some(Display::Vouched));
        update.update_profile(&mut p, &secret_store, &fossil_settings)?;
        assert_eq!(p.pronouns.value, None);
        assert_eq!(p.fun_title.value, Some(String::default()));
        assert_eq!(p.fun_title.metadata.display, Some(Display::Vouched));
        Ok(())
    }

    #[test]
    fn test_update_display_only_with_null_value_kv() -> Result<(), Error> {
        let secret_store = get_fake_secret_store();
        let fossil_settings = Fossil {
            upload_endpoint: String::default(),
        };
        let mut p = Profile::default();
        let mut update = InputProfile::default();
        update.languages = Some(KeyValuesWithDisplay {
            values: None,
            display: Some(Display::Vouched),
        });
        assert_eq!(p.tags.values, None);
        assert_eq!(p.languages.values, None);
        assert_ne!(p.languages.metadata.display, Some(Display::Vouched));
        update.update_profile(&mut p, &secret_store, &fossil_settings)?;
        assert_eq!(p.tags.values, None);
        assert_eq!(p.languages.values, Some(Default::default()));
        assert_eq!(p.languages.metadata.display, Some(Display::Vouched));
        Ok(())
    }
}
