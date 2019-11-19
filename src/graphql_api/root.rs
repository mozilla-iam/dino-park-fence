use crate::graphql_api::error::field_error;
use crate::graphql_api::input::InputProfile;
use crate::metrics::Metrics;
use crate::settings::DinoParkServices;
use actix_web::test;
use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use cis_profile::schema::Display;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;
use log::error;
use log::info;
use log::warn;
use reqwest::Client;

pub struct Query<T: AsyncCisClientTrait> {
    pub cis_client: T,
    pub dinopark_settings: DinoParkServices,
}

fn get_profile(
    id: String,
    cis_client: &impl AsyncCisClientTrait,
    by: &GetBy,
    filter: &str,
) -> FieldResult<Profile> {
    test::block_on(cis_client.get_user_by(&id, by, Some(&filter))).map_err(Into::into)
}

pub struct Mutation<T: AsyncCisClientTrait> {
    pub cis_client: T,
    pub dinopark_settings: DinoParkServices,
}

fn update_profile(
    update: InputProfile,
    cis_client: &impl AsyncCisClientTrait,
    dinopark_settings: &DinoParkServices,
    user: &Option<String>,
) -> FieldResult<(Profile, bool)> {
    let user_id = user
        .clone()
        .ok_or_else(|| field_error("no username in query or scope", "?!"))?;
    let mut profile = test::block_on(cis_client.get_user_by(&user_id, &GetBy::UserId, None))?;
    if let Some(updated_username) = update
        .primary_username
        .as_ref()
        .and_then(|s| s.value.as_ref())
    {
        if Some(updated_username) != profile.primary_username.value.as_ref() {
            let num_chars = updated_username.chars().count();
            if num_chars < 2 || num_chars > 64 {
                return Err(field_error(
                    "username_length",
                    "Lenght of username must be between 2 and 64. And only contain letters from a-z, digits from 0-9, underscore or hyphen.",
                ));
            }
            let only_valid_chars = updated_username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if !only_valid_chars {
                return Err(field_error(
                    "username_invalid_chars",
                    "Lenght of username must be between 2 and 64. And only contain letters from a-z, digits from 0-9, underscore or hyphen.",
                ));
            }
            // the primary_username changed check if it already exists
            if test::block_on(cis_client.get_user_by(
                updated_username,
                &GetBy::PrimaryUsername,
                None,
            ))
            .is_ok()
            {
                return Err(field_error(
                    "username_exists",
                    "This username already exitst!",
                ));
            }
        }
    }

    let changed = update
        .update_profile(
            &mut profile,
            cis_client.get_secret_store(),
            &dinopark_settings.fossil,
        )
        .map_err(|e| field_error("unable update/sign profle", e))?;
    if changed {
        let ret = test::block_on(cis_client.update_user(&user_id, profile))?;
        info!("update returned: {}", ret);
        let updated_profile =
            test::block_on(cis_client.get_user_by(&user_id, &GetBy::UserId, None))?;
        if dinopark_settings.lookout.internal_update_enabled {
            if let Err(e) = Client::new()
                .post(&dinopark_settings.lookout.internal_update_endpoint)
                .json(&updated_profile)
                .send()
            {
                error!("unable to post to lookout: {}", e);
            }
        }
        Ok((updated_profile, changed))
    } else {
        Ok((profile, changed))
    }
}

#[juniper::object{
    Context = (ScopeAndUser, Metrics)
}]
impl<T: AsyncCisClientTrait> Query<T> {
    fn profile(username: Option<String>, view_as: Option<Display>) -> FieldResult<Profile> {
        let executor = &executor;
        let scope_and_user = &executor.context().0;

        let params = get_profile_params(username, scope_and_user, view_as)?;

        get_profile(
            params.id,
            &self.cis_client,
            &params.by,
            params.filter.as_str(),
        )
    }
}

#[juniper::object{
    Context = (ScopeAndUser, Metrics)
}]
impl<T: AsyncCisClientTrait> Mutation<T> {
    fn profile(update: InputProfile) -> FieldResult<Profile> {
        let executor = &executor;
        match update_profile(
            update,
            &self.cis_client,
            &self.dinopark_settings,
            &Some(executor.context().0.user_id.clone()),
        ) {
            Ok((profile, true)) => {
                executor.context().1.counters.field_any_changed.inc();
                Ok(profile)
            }
            Ok((profile, false)) => Ok(profile),
            Err(e) => Err(e),
        }
    }
}

pub type Schema<T> = RootNode<'static, Query<T>, Mutation<T>>;

struct GetProfileParams {
    id: String,
    by: GetBy,
    filter: Display,
}

fn get_profile_params(
    username: Option<String>,
    scope_and_user: &ScopeAndUser,
    view_as: Option<Display>,
) -> Result<GetProfileParams, FieldError> {
    let scope = match serde_json::from_value(scope_and_user.scope.clone().into()) {
        Ok(scope) => scope,
        Err(e) => {
            warn!(
                "invalid scope {} for {}: {}",
                scope_and_user.scope, scope_and_user.user_id, e
            );
            Display::Public
        }
    };
    let params = if let Some(username) = username {
        // If a username has been provided we retrieve the
        // profile by username and filter according to
        // view_as if view_as if less restrictive than the
        // users scope.
        let filter = if let Some(filter) = view_as {
            if filter <= scope {
                filter
            } else {
                warn!(
                    "invalid display {} for {} ({})",
                    filter.as_str(),
                    scope_and_user.user_id,
                    scope_and_user.scope
                );
                return Err(field_error(
                    "invalid_view_as",
                    "Insufficient permission for requested view_as display level!",
                ));
            }
        } else {
            scope
        };
        GetProfileParams {
            id: username,
            by: GetBy::PrimaryUsername,
            filter,
        }
    } else {
        // If no username has been provided we retrieve the
        // profile by user_id of the current user and allow
        // any view as if provided, otherwise use private.
        GetProfileParams {
            id: scope_and_user.user_id.clone(),
            by: GetBy::UserId,
            filter: view_as.unwrap_or_else(|| Display::Private),
        }
    };
    Ok(params)
}

#[cfg(test)]
mod root_test {
    use super::*;

    #[test]
    fn test_get_filter_params_without_view_as() -> Result<(), FieldError> {
        let username = Some(String::from("user1"));
        let scope_and_user = ScopeAndUser {
            user_id: String::from("user2"),
            scope: String::from("staff"),
            groups_scope: None,
        };
        let params = get_profile_params(username, &scope_and_user, None)?;
        assert!(match params.by {
            GetBy::PrimaryUsername => true,
            _ => false,
        });
        assert_eq!(params.id, "user1");
        assert_eq!(params.filter, Display::Staff);
        Ok(())
    }

    #[test]
    fn test_get_filter_params_with_view_as_pass() -> Result<(), FieldError> {
        let username = Some(String::from("user1"));
        let view_as = Some(Display::Ndaed);
        let scope_and_user = ScopeAndUser {
            user_id: String::from("user2"),
            scope: String::from("staff"),
            groups_scope: None,
        };
        let params = get_profile_params(username, &scope_and_user, view_as)?;
        assert!(match params.by {
            GetBy::PrimaryUsername => true,
            _ => false,
        });
        assert_eq!(params.id, "user1");
        assert_eq!(params.filter, Display::Ndaed);
        Ok(())
    }

    #[test]
    fn test_get_filter_params_with_view_as_fail() -> Result<(), FieldError> {
        let username = Some(String::from("user1"));
        let view_as = Some(Display::Ndaed);
        let scope_and_user = ScopeAndUser {
            user_id: String::from("user2"),
            scope: String::from("authenticated"),
            groups_scope: None,
        };
        let params = get_profile_params(username, &scope_and_user, view_as);
        assert!(params.is_err());
        Ok(())
    }

    #[test]
    fn test_get_filter_params_self_without_view_as() -> Result<(), FieldError> {
        let scope_and_user = ScopeAndUser {
            user_id: String::from("user1"),
            scope: String::from("staff"),
            groups_scope: None,
        };
        let params = get_profile_params(None, &scope_and_user, None)?;
        assert!(match params.by {
            GetBy::UserId => true,
            _ => false,
        });
        assert_eq!(params.id, "user1");
        assert_eq!(params.filter, Display::Private);
        Ok(())
    }

    #[test]
    fn test_get_filter_params_self_with_higher_view_as() -> Result<(), FieldError> {
        let view_as = Some(Display::Staff);
        let scope_and_user = ScopeAndUser {
            user_id: String::from("user1"),
            scope: String::from("authenticated"),
            groups_scope: None,
        };
        let params = get_profile_params(None, &scope_and_user, view_as)?;
        assert!(match params.by {
            GetBy::UserId => true,
            _ => false,
        });
        assert_eq!(params.id, "user1");
        assert_eq!(params.filter, Display::Staff);
        Ok(())
    }
}
