use crate::graphql_api::error::field_error;
use crate::graphql_api::input::InputProfile;
use crate::metrics::Metrics;
use crate::settings::DinoParkServices;
use cis_client::error::ProfileError;
use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use cis_profile::schema::Display;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use dino_park_trust::Trust;
use failure::Error;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;
use log::error;
use log::info;
use log::warn;
use reqwest::Client;
use std::sync::Arc;

const INVALID_USERNAME_MESSAGE: &str = "\
Length of username must be between 2 and 64. \
And only contain lowercase letters from a-z, digits from 0-9, underscore or hyphen.";

pub struct Query<T: AsyncCisClientTrait> {
    pub cis_client: T,
    pub dinopark_settings: DinoParkServices,
}

async fn get_profile(
    id: String,
    cis_client: &impl AsyncCisClientTrait,
    by: &GetBy,
    filter: &str,
) -> Result<Profile, Error> {
    cis_client.get_user_by(&id, by, Some(&filter)).await
}

pub struct Mutation<T: AsyncCisClientTrait> {
    pub cis_client: T,
    pub dinopark_settings: DinoParkServices,
}

fn valid_username(username: &str) -> Result<(), FieldError> {
    let num_chars = username.chars().count();
    if !(2..=64).contains(&num_chars) {
        return Err(field_error("username_length", INVALID_USERNAME_MESSAGE));
    }
    let only_valid_chars = username
        .chars()
        .all(|c| (c.is_ascii_lowercase() || c.is_ascii_digit()) || c == '-' || c == '_');
    if !only_valid_chars {
        return Err(field_error(
            "username_invalid_chars",
            INVALID_USERNAME_MESSAGE,
        ));
    }
    Ok(())
}

async fn update_profile(
    update: InputProfile,
    cis_client: &impl AsyncCisClientTrait,
    dinopark_settings: &DinoParkServices,
    user: &Option<String>,
    scope: Trust,
) -> FieldResult<(Profile, bool)> {
    let user_id = user
        .clone()
        .ok_or_else(|| field_error("no username in query or scope", "?!"))?;
    let mut profile = cis_client
        .get_user_by(&user_id, &GetBy::UserId, None)
        .await?;
    if let Some(updated_username) = update
        .primary_username
        .as_ref()
        .and_then(|s| s.value.as_ref())
    {
        if Some(updated_username) != profile.primary_username.value.as_ref() {
            valid_username(&updated_username)?;
            // the primary_username changed check if it already exists
            if cis_client
                .get_any_user_by(updated_username, &GetBy::PrimaryUsername, None)
                .await
                .is_ok()
            {
                return Err(field_error(
                    "username_exists",
                    "This username already exists!",
                ));
            }
        }
    }

    let changed = update
        .update_profile(
            &mut profile,
            &scope,
            cis_client.get_secret_store(),
            &dinopark_settings.fossil,
        )
        .await
        .map_err(|e| field_error("unable update/sign profile", e))?;
    if changed {
        let ret = cis_client.update_user(&user_id, profile).await?;
        info!("update returned: {}", ret);
        let updated_profile = cis_client
            .get_user_by(&user_id, &GetBy::UserId, None)
            .await?;
        if dinopark_settings.lookout.internal_update_enabled {
            if let Err(e) = Client::new()
                .post(&dinopark_settings.lookout.internal_update_endpoint)
                .json(&updated_profile)
                .send()
                .await
            {
                error!("unable to post to lookout: {}", e);
            }
        }
        Ok((updated_profile, changed))
    } else {
        Ok((profile, changed))
    }
}

#[juniper::graphql_object{
    Context = (ScopeAndUser, Arc<Metrics>)
}]
impl<T: AsyncCisClientTrait + Send + Sync> Query<T> {
    async fn profile(username: Option<String>, view_as: Option<Display>) -> FieldResult<Profile> {
        let self_query = username.is_none();
        let executor = &executor;
        let scope_and_user = &executor.context().0;
        if scope_and_user.scope == Trust::Public && self_query {
            return Ok(Profile::default());
        }

        let params = get_profile_params(username, scope_and_user, view_as)?;

        match get_profile(
            params.id,
            &self.cis_client,
            &params.by,
            params.filter.as_str(),
        )
        .await
        {
            Ok(p) => Ok(p),
            Err(e) if self_query => match e.downcast::<ProfileError>() {
                Ok(ProfileError::ProfileDoesNotExist) => Err(FieldError::new(
                    "wait_for_profile",
                    graphql_value!({"warning": "profile does not exist yet"}),
                )),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }
    }
}

#[juniper::graphql_object{
    Context = (ScopeAndUser, Arc<Metrics>)
}]
impl<T: AsyncCisClientTrait + Send + Sync> Mutation<T> {
    async fn profile(update: InputProfile) -> FieldResult<Profile> {
        let executor = &executor;
        let scope_and_user = &executor.context().0;
        if scope_and_user.scope == Trust::Public {
            return Ok(Profile::default());
        }
        match update_profile(
            update,
            &self.cis_client,
            &self.dinopark_settings,
            &Some(executor.context().0.user_id.clone()),
            executor.context().0.scope.clone(),
        )
        .await
        {
            Ok((profile, true)) => {
                executor.context().1.counters.field_any_changed.inc();
                Ok(profile)
            }
            Ok((profile, false)) => Ok(profile),
            Err(e) => Err(e),
        }
    }
}

pub type Schema<T> = RootNode<
    'static,
    Query<T>,
    Mutation<T>,
    juniper::EmptySubscription<(ScopeAndUser, Arc<Metrics>)>,
>;

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
    let scope: Display = scope_and_user.scope.clone().into();
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
                    scope_and_user.scope.as_str()
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
            filter: view_as.unwrap_or(Display::Private),
        }
    };
    Ok(params)
}

#[cfg(test)]
mod root_test {
    use super::*;
    use dino_park_trust::AALevel;
    use dino_park_trust::GroupsTrust;
    use dino_park_trust::Trust;

    #[test]
    fn test_get_filter_params_without_view_as() -> Result<(), FieldError> {
        let username = Some(String::from("user1"));
        let scope_and_user = ScopeAndUser {
            user_id: String::from("user2"),
            scope: Trust::Staff,
            groups_scope: GroupsTrust::None,
            aa_level: AALevel::Low,
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
            scope: Trust::Staff,
            groups_scope: GroupsTrust::None,
            aa_level: AALevel::Low,
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
            scope: Trust::Authenticated,
            groups_scope: GroupsTrust::None,
            aa_level: AALevel::Low,
        };
        let params = get_profile_params(username, &scope_and_user, view_as);
        assert!(params.is_err());
        Ok(())
    }

    #[test]
    fn test_get_filter_params_self_without_view_as() -> Result<(), FieldError> {
        let scope_and_user = ScopeAndUser {
            user_id: String::from("user1"),
            scope: Trust::Staff,
            groups_scope: GroupsTrust::None,
            aa_level: AALevel::Low,
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
            scope: Trust::Authenticated,
            groups_scope: GroupsTrust::None,
            aa_level: AALevel::Low,
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

    #[test]
    fn test_username() {
        assert!(valid_username("r--lwqkc13jeqw314").is_ok());
        assert!(valid_username("--__--").is_ok());
        assert!(valid_username("r--A").is_err());
        assert!(valid_username("r--🦊").is_err());
        assert!(valid_username("").is_err());
        assert!(valid_username("a").is_err());
        assert!(valid_username("a".repeat(65).as_str()).is_err());
    }
}
