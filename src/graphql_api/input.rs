use crate::graphql_api::avatar::change_picture_display;
use crate::graphql_api::avatar::save_picture;
use crate::settings::Fossil;
use chrono::DateTime;
use chrono::Utc;
use cis_profile::crypto::Signer;
use cis_profile::schema::AccessInformationProviderSubObject;
use cis_profile::schema::Display;
use cis_profile::schema::IdentitiesAttributesValuesArray;
use cis_profile::schema::KeyValue;
use cis_profile::schema::Profile;
use cis_profile::schema::PublisherAuthority;
use cis_profile::schema::StandardAttributeString;
use cis_profile::schema::StandardAttributeValues;
use dino_park_trust::Trust;
use failure::format_err;
use failure::Error;
use juniper::GraphQLInputObject;
use std::collections::BTreeMap;

const DISPLAY_ANY: &[Display; 6] = &[
    Display::Private,
    Display::Staff,
    Display::Ndaed,
    Display::Vouched,
    Display::Authenticated,
    Display::Public,
];

const DISPLAY_NOT_PRIVATE: &[Display; 5] = &[
    Display::Staff,
    Display::Ndaed,
    Display::Vouched,
    Display::Authenticated,
    Display::Public,
];

const DISPLAY_PRIVATE_STAFF: &[Display; 2] = &[Display::Private, Display::Staff];

fn create_usernames_key(typ: &str) -> String {
    format!("HACK#{}", typ)
}

fn update_access_information_display(
    d: &Option<Display>,
    p: &mut AccessInformationProviderSubObject,
    now: &DateTime<Utc>,
    store: &impl Signer,
    allowed: &[Display],
) -> Result<bool, Error> {
    if *d != p.metadata.display {
        if let Some(display) = &d {
            if !allowed.contains(display) {
                return Err(format_err!("invalid display level"));
            }
            // Initialize with empty values if there are now access groups.
            if p.values.is_none() {
                p.values = Some(KeyValue(BTreeMap::default()));
            }
            p.metadata.display = Some(display.clone());
            p.metadata.last_modified = *now;
            p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            store.sign_attribute(p)?;
            return Ok(true);
        }
    }
    Ok(false)
}

async fn update_picture(
    s: &Option<StringWithDisplay>,
    p: &mut StandardAttributeString,
    uuid: &StandardAttributeString,
    now: &DateTime<Utc>,
    store: &impl Signer,
    fossil_settings: &Fossil,
) -> Result<bool, Error> {
    let mut changed = false;
    if let Some(new_picture) = s {
        if new_picture.display != p.metadata.display {
            if let Some(display) = &new_picture.display {
                if !DISPLAY_NOT_PRIVATE.contains(display) {
                    return Err(format_err!("invalid display level"));
                }
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
                    let url = save_picture(
                        value,
                        uuid,
                        display,
                        p.value.as_deref(),
                        &fossil_settings.upload_endpoint,
                    )
                    .await?;
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
                    display,
                    p.value.as_deref(),
                    &fossil_settings.upload_endpoint,
                )
                .await?;
                p.value = Some(url);
                changed = true;
            }
        } else if new_picture.value != p.value && new_picture.value == Some(String::default()) {
            // TODO: delete picture
            p.value = new_picture.value.clone();
            changed = true;
        }

        if changed {
            p.metadata.last_modified = *now;
            p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            store.sign_attribute(p)?;
        }
    }
    Ok(changed)
}

fn update_google_identity(
    google: &IdentityWithDisplay,
    p: &mut IdentitiesAttributesValuesArray,
    now: &DateTime<Utc>,
    store: &impl Signer,
) -> Result<bool, Error> {
    let mut changed_google = false;
    if google.remove.unwrap_or_default() {
        p.google_oauth2_id.metadata.display = Some(Display::Staff);
        p.google_primary_email.metadata.display = Some(Display::Staff);

        p.google_oauth2_id.value = Some(String::default());
        p.google_primary_email.value = Some(String::default());
        changed_google = true;
    } else if google.display != p.google_oauth2_id.metadata.display
        || google.display != p.google_primary_email.metadata.display
    {
        if let Some(display) = &google.display {
            if !DISPLAY_NOT_PRIVATE.contains(display) {
                return Err(format_err!("invalid display level"));
            }
            if p.google_oauth2_id.value.is_none() {
                p.google_oauth2_id.value = Some(String::default())
            }
            if p.google_primary_email.value.is_none() {
                p.google_primary_email.value = Some(String::default())
            }

            p.google_oauth2_id.metadata.display = Some(display.clone());
            p.google_primary_email.metadata.display = Some(display.clone());
            changed_google = true;
        }
    }

    if changed_google {
        p.google_oauth2_id.metadata.last_modified = *now;
        p.google_primary_email.metadata.last_modified = now.to_owned();
        p.google_oauth2_id.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        p.google_primary_email.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(&mut p.google_oauth2_id)?;
        store.sign_attribute(&mut p.google_primary_email)?;
    }

    Ok(changed_google)
}

fn update_bugzilla_identity(
    bugzilla: &IdentityWithDisplay,
    p: &mut IdentitiesAttributesValuesArray,
    u: &mut StandardAttributeValues,
    now: &DateTime<Utc>,
    store: &impl Signer,
) -> Result<bool, Error> {
    let mut changed_bugzilla = false;
    let mut changed_usernames = false;
    if bugzilla.remove.unwrap_or_default() {
        p.bugzilla_mozilla_org_id.metadata.display = Some(Display::Staff);
        p.bugzilla_mozilla_org_primary_email.metadata.display = Some(Display::Staff);

        if let Some(KeyValue(usernames)) = &mut u.values {
            if usernames.remove(&create_usernames_key("BMOMAIL")).is_some() {
                changed_usernames = true;
            }
            if usernames.remove(&create_usernames_key("BMONICK")).is_some() {
                changed_usernames = true;
            }
        }

        p.bugzilla_mozilla_org_id.value = Some(String::default());
        p.bugzilla_mozilla_org_primary_email.value = Some(String::default());
        changed_bugzilla = true;
    } else if bugzilla.display != p.bugzilla_mozilla_org_id.metadata.display
        || bugzilla.display != p.bugzilla_mozilla_org_primary_email.metadata.display
    {
        if let Some(display) = &bugzilla.display {
            if !DISPLAY_NOT_PRIVATE.contains(display) {
                return Err(format_err!("invalid display level"));
            }
            if p.bugzilla_mozilla_org_id.value.is_none() {
                p.bugzilla_mozilla_org_id.value = Some(String::default())
            }
            if p.bugzilla_mozilla_org_primary_email.value.is_none() {
                p.bugzilla_mozilla_org_primary_email.value = Some(String::default())
            }

            p.bugzilla_mozilla_org_id.metadata.display = Some(display.clone());
            p.bugzilla_mozilla_org_primary_email.metadata.display = Some(display.clone());
            changed_bugzilla = true;
        }
    }

    if changed_bugzilla {
        p.bugzilla_mozilla_org_id.metadata.last_modified = *now;
        p.bugzilla_mozilla_org_primary_email.metadata.last_modified = now.to_owned();
        p.bugzilla_mozilla_org_id.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        p.bugzilla_mozilla_org_primary_email
            .signature
            .publisher
            .name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(&mut p.bugzilla_mozilla_org_id)?;
        store.sign_attribute(&mut p.bugzilla_mozilla_org_primary_email)?;
    }

    if changed_usernames {
        u.metadata.last_modified = *now;
        u.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(u)?;
    }

    Ok(changed_bugzilla || changed_usernames)
}

fn update_github_identity(
    github: &IdentityWithDisplay,
    p: &mut IdentitiesAttributesValuesArray,
    u: &mut StandardAttributeValues,
    now: &DateTime<Utc>,
    store: &impl Signer,
) -> Result<bool, Error> {
    let mut changed_github = false;
    let mut changed_usernames = false;
    if github.remove.unwrap_or_default() {
        p.github_id_v3.metadata.display = Some(Display::Staff);
        p.github_id_v4.metadata.display = Some(Display::Staff);
        p.github_primary_email.metadata.display = Some(Display::Staff);

        if let Some(KeyValue(usernames)) = &mut u.values {
            if usernames.remove(&create_usernames_key("GITHUB")).is_some() {
                changed_usernames = true;
            }
        }

        p.github_id_v3.value = Some(String::default());
        p.github_id_v4.value = Some(String::default());
        p.github_primary_email.value = Some(String::default());
        changed_github = true;
    } else if github.display != p.github_id_v3.metadata.display
        || github.display != p.github_id_v4.metadata.display
        || github.display != p.github_primary_email.metadata.display
    {
        if let Some(display) = &github.display {
            if !DISPLAY_NOT_PRIVATE.contains(display) {
                return Err(format_err!("invalid display level"));
            }
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
            changed_github = true;
        }
    }

    if changed_github {
        p.github_id_v3.metadata.last_modified = *now;
        p.github_id_v4.metadata.last_modified = *now;
        p.github_primary_email.metadata.last_modified = now.to_owned();
        p.github_id_v3.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        p.github_id_v4.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        p.github_primary_email.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(&mut p.github_id_v3)?;
        store.sign_attribute(&mut p.github_id_v4)?;
        store.sign_attribute(&mut p.github_primary_email)?;
    }

    if changed_usernames {
        u.metadata.last_modified = *now;
        u.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(u)?;
    }

    Ok(changed_github || changed_usernames)
}

fn update_identities(
    i: &Option<IdentitiesWithDisplay>,
    p: &mut IdentitiesAttributesValuesArray,
    u: &mut StandardAttributeValues,
    now: &DateTime<Utc>,
    store: &impl Signer,
) -> Result<bool, Error> {
    let mut changed = false;
    if let Some(identities) = i {
        if let Some(github) = &identities.github {
            changed |= update_github_identity(github, p, u, now, store)?;
        }
        if let Some(bugzilla) = &identities.bugzilla {
            changed |= update_bugzilla_identity(bugzilla, p, u, now, store)?;
        }
        if let Some(google) = &identities.google {
            changed |= update_google_identity(google, p, now, store)?;
        }
    }

    Ok(changed)
}

fn update_display_for_string(
    d: &Option<Display>,
    p: &mut StandardAttributeString,
    now: &DateTime<Utc>,
    store: &impl Signer,
    allowed: &[Display],
) -> Result<bool, Error> {
    let mut changed = false;
    if d != &p.metadata.display {
        if let Some(display) = &d {
            if !allowed.contains(display) {
                return Err(format_err!("invalid display level"));
            }
            // if display changed but field is null we cannot do anything
            if p.value.is_some() {
                p.metadata.display = Some(display.clone());
                changed = true;
            }
        }
    }
    if changed {
        p.metadata.last_modified = *now;
        p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(p)?;
    }
    Ok(changed)
}

fn update_display_for_key_values(
    d: &Option<Display>,
    p: &mut StandardAttributeValues,
    now: &DateTime<Utc>,
    store: &impl Signer,
    allowed: &[Display],
) -> Result<bool, Error> {
    let mut changed = false;
    if d != &p.metadata.display {
        if let Some(display) = &d {
            if !allowed.contains(display) {
                return Err(format_err!("invalid display level"));
            }
            // if display changed but field is null change it to empty string
            if p.values.is_some() {
                p.metadata.display = Some(display.clone());
                changed = true;
            }
        }
    }
    if changed {
        p.metadata.last_modified = *now;
        p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
        store.sign_attribute(p)?;
    }
    Ok(changed)
}

fn update_string(
    s: &Option<StringWithDisplay>,
    p: &mut StandardAttributeString,
    now: &DateTime<Utc>,
    store: &impl Signer,
    allowed: &[Display],
) -> Result<bool, Error> {
    let mut changed = false;
    if let Some(x) = s {
        if x.value != p.value {
            if let Some(value) = &x.value {
                p.value = Some(value.clone());
                changed = true;
            }
        }
        if x.display != p.metadata.display {
            if let Some(display) = &x.display {
                if !allowed.contains(display) {
                    return Err(format_err!("invalid display level"));
                }
                // if display changed but field is null change it to empty string
                if p.value.is_none() {
                    p.value = Some(String::default());
                }
                p.metadata.display = Some(display.clone());
                changed = true;
            }
        }
        if changed {
            p.metadata.last_modified = *now;
            p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            store.sign_attribute(p)?;
        }
    }
    Ok(changed)
}

fn update_key_values(
    s: &Option<KeyValuesWithDisplay>,
    p: &mut StandardAttributeValues,
    now: &DateTime<Utc>,
    store: &impl Signer,
    filter_empty_values: bool,
    allowed: &[Display],
) -> Result<bool, Error> {
    let mut changed = false;
    if let Some(x) = s {
        if let Some(values) = &x.values {
            let values: BTreeMap<String, Option<String>> = if filter_empty_values {
                values
                    .iter()
                    .filter_map(|e| {
                        if e.v.as_ref().map(|s| s.is_empty()).unwrap_or_default() {
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
            changed = true;
        }
        if x.display != p.metadata.display {
            if let Some(display) = &x.display {
                if !allowed.contains(display) {
                    return Err(format_err!("invalid display level"));
                }
                // if display changed but field is null change it to empty dict
                if p.values.is_none() {
                    p.values = Some(KeyValue(BTreeMap::default()));
                }
                p.metadata.display = Some(display.clone());
                changed = true;
            }
        }
        if changed {
            p.metadata.last_modified = *now;
            p.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            store.sign_attribute(p)?;
        }
    }
    Ok(changed)
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
    pub google: Option<IdentityWithDisplay>,
}

#[derive(GraphQLInputObject, Default)]
pub struct InputProfile {
    pub access_information_ldap_display: Option<Display>,
    // TODO: delete after upgrade
    pub access_information_mozilliansorg: Option<Display>,
    pub access_information_mozilliansorg_display: Option<Display>,
    pub active: Option<BoolWithDisplay>,
    pub alternative_name: Option<StringWithDisplay>,
    pub created: Option<StringWithDisplay>,
    pub custom_1_primary_email: Option<StringWithDisplay>,
    pub custom_2_primary_email: Option<StringWithDisplay>,
    pub description: Option<StringWithDisplay>,
    pub first_name: Option<StringWithDisplay>,
    pub fun_title: Option<StringWithDisplay>,
    pub identities: Option<IdentitiesWithDisplay>,
    pub languages: Option<KeyValuesWithDisplay>,
    pub last_modified: Option<StringWithDisplay>,
    pub last_name: Option<StringWithDisplay>,
    pub location: Option<StringWithDisplay>,
    pub login_method: Option<StringWithDisplay>,
    pub pgp_public_keys_display: Option<Display>,
    pub phone_numbers: Option<KeyValuesWithDisplay>,
    pub picture: Option<StringWithDisplay>,
    pub primary_email_display: Option<Display>,
    pub primary_username: Option<StringWithDisplay>,
    pub pronouns: Option<StringWithDisplay>,
    pub ssh_public_keys_display: Option<Display>,
    pub staff_information_title_display: Option<Display>,
    pub staff_information_office_location_display: Option<Display>,
    pub tags: Option<KeyValuesWithDisplay>,
    pub timezone: Option<StringWithDisplay>,
    pub uris: Option<KeyValuesWithDisplay>,
    pub user_id: Option<StringWithDisplay>,
    pub usernames: Option<KeyValuesWithDisplay>,
}

impl InputProfile {
    pub async fn update_profile(
        &self,
        p: &mut Profile,
        scope: &Trust,
        secret_store: &impl Signer,
        fossil_settings: &Fossil,
    ) -> Result<bool, Error> {
        let now = &Utc::now();
        let mut changed = false;
        changed |= update_string(
            &self.alternative_name,
            &mut p.alternative_name,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.created,
            &mut p.created,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.custom_1_primary_email,
            &mut p.identities.custom_1_primary_email,
            now,
            secret_store,
            DISPLAY_ANY,
        )?;
        changed |= update_string(
            &self.custom_2_primary_email,
            &mut p.identities.custom_2_primary_email,
            now,
            secret_store,
            DISPLAY_ANY,
        )?;
        changed |= update_string(
            &self.description,
            &mut p.description,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.first_name,
            &mut p.first_name,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.fun_title,
            &mut p.fun_title,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.last_modified,
            &mut p.last_modified,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.last_name,
            &mut p.last_name,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.location,
            &mut p.location,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.login_method,
            &mut p.login_method,
            now,
            secret_store,
            DISPLAY_ANY,
        )?;
        changed |= update_picture(
            &self.picture,
            &mut p.picture,
            &p.uuid,
            now,
            secret_store,
            fossil_settings,
        )
        .await?;
        changed |= update_display_for_string(
            &self.primary_email_display,
            &mut p.primary_email,
            now,
            secret_store,
            if scope == &Trust::Staff {
                DISPLAY_NOT_PRIVATE
            } else {
                DISPLAY_ANY
            },
        )?;
        changed |= update_string(
            &self.primary_username,
            &mut p.primary_username,
            now,
            secret_store,
            &[Display::Public],
        )?;
        changed |= update_string(
            &self.pronouns,
            &mut p.pronouns,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.timezone,
            &mut p.timezone,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_string(
            &self.user_id,
            &mut p.user_id,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;

        changed |= update_key_values(
            &self.languages,
            &mut p.languages,
            now,
            secret_store,
            false,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_key_values(
            &self.phone_numbers,
            &mut p.phone_numbers,
            now,
            secret_store,
            true,
            DISPLAY_ANY,
        )?;
        changed |= update_key_values(
            &self.tags,
            &mut p.tags,
            now,
            secret_store,
            false,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_key_values(
            &self.usernames,
            &mut p.usernames,
            now,
            secret_store,
            true,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_key_values(
            &self.uris,
            &mut p.uris,
            now,
            secret_store,
            true,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_display_for_key_values(
            &self.pgp_public_keys_display,
            &mut p.pgp_public_keys,
            now,
            secret_store,
            DISPLAY_ANY,
        )?;
        changed |= update_display_for_key_values(
            &self.ssh_public_keys_display,
            &mut p.ssh_public_keys,
            now,
            secret_store,
            DISPLAY_ANY,
        )?;
        changed |= update_identities(
            &self.identities,
            &mut p.identities,
            &mut p.usernames,
            now,
            secret_store,
        )?;

        // TODO: delete after upgrade
        changed |= update_access_information_display(
            &self.access_information_mozilliansorg,
            &mut p.access_information.mozilliansorg,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_access_information_display(
            &self.access_information_mozilliansorg_display,
            &mut p.access_information.mozilliansorg,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_access_information_display(
            &self.access_information_ldap_display,
            &mut p.access_information.ldap,
            now,
            secret_store,
            DISPLAY_PRIVATE_STAFF,
        )?;
        changed |= update_display_for_string(
            &self.staff_information_title_display,
            &mut p.staff_information.title,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        changed |= update_display_for_string(
            &self.staff_information_office_location_display,
            &mut p.staff_information.office_location,
            now,
            secret_store,
            DISPLAY_NOT_PRIVATE,
        )?;
        Ok(changed)
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

    #[tokio::test]
    async fn test_simple_update() -> Result<(), Error> {
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
        update
            .update_profile(&mut p, &Trust::Staff, &secret_store, &fossil_settings)
            .await?;
        assert_eq!(p.fun_title.value, update.fun_title.unwrap().value);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_with_invalid_display_fails() -> Result<(), Error> {
        let secret_store = get_fake_secret_store();
        let fossil_settings = Fossil {
            upload_endpoint: String::default(),
        };
        let mut p = Profile::default();
        let mut update = InputProfile::default();
        update.fun_title = Some(StringWithDisplay {
            value: None,
            display: Some(Display::Private),
        });
        assert_eq!(p.pronouns.value, None);
        assert_eq!(p.fun_title.value, None);
        assert_ne!(p.fun_title.metadata.display, Some(Display::Private));
        assert!(update
            .update_profile(&mut p, &Trust::Staff, &secret_store, &fossil_settings)
            .await
            .is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_update_display_only_with_null_value_string() -> Result<(), Error> {
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
        update
            .update_profile(&mut p, &Trust::Staff, &secret_store, &fossil_settings)
            .await?;
        assert_eq!(p.pronouns.value, None);
        assert_eq!(p.fun_title.value, Some(String::default()));
        assert_eq!(p.fun_title.metadata.display, Some(Display::Vouched));
        Ok(())
    }

    #[tokio::test]
    async fn test_update_display_only_with_null_value_kv() -> Result<(), Error> {
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
        update
            .update_profile(&mut p, &Trust::Staff, &secret_store, &fossil_settings)
            .await?;
        assert_eq!(p.tags.values, None);
        assert_eq!(p.languages.values, Some(Default::default()));
        assert_eq!(p.languages.metadata.display, Some(Display::Vouched));
        Ok(())
    }

    #[tokio::test]
    async fn test_update_access_information_display_initializes_groups() -> Result<(), Error> {
        let secret_store = get_fake_secret_store();
        let fossil_settings = Fossil {
            upload_endpoint: String::default(),
        };
        let mut p = Profile::default();
        let update = InputProfile {
            access_information_mozilliansorg_display: Some(Display::Ndaed),
            ..Default::default()
        };
        assert_eq!(p.access_information.mozilliansorg.values, None);
        assert_ne!(
            p.access_information.mozilliansorg.metadata.display,
            Some(Display::Ndaed)
        );
        update
            .update_profile(&mut p, &Trust::Staff, &secret_store, &fossil_settings)
            .await?;
        assert_eq!(
            p.access_information.mozilliansorg.values,
            Some(KeyValue(BTreeMap::default()))
        );
        assert_eq!(
            p.access_information.mozilliansorg.metadata.display,
            Some(Display::Ndaed)
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_update_access_information_display_keeps_groups() -> Result<(), Error> {
        let secret_store = get_fake_secret_store();
        let fossil_settings = Fossil {
            upload_endpoint: String::default(),
        };

        let mut groups = BTreeMap::new();
        groups.insert(String::from("Something"), None);

        let mut p = Profile::default();
        p.access_information.mozilliansorg.values = Some(KeyValue(groups.clone()));
        p.access_information.mozilliansorg.metadata.display = Some(Display::Private);

        let update = InputProfile {
            access_information_mozilliansorg_display: Some(Display::Ndaed),
            ..Default::default()
        };
        update
            .update_profile(&mut p, &Trust::Staff, &secret_store, &fossil_settings)
            .await?;
        assert_eq!(
            p.access_information.mozilliansorg.values,
            Some(KeyValue(groups))
        );
        assert_eq!(
            p.access_information.mozilliansorg.metadata.display,
            Some(Display::Ndaed)
        );
        Ok(())
    }
}
