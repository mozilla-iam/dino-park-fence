use crate::graphql_api::input::InputProfile;
use crate::permissions::Scope;
use crate::permissions::UserId;
use crate::settings::DinoParkServices;
use actix_web::test;
use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use cis_profile::schema::Profile;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;
use reqwest::Client;

pub struct Query<T: AsyncCisClientTrait> {
    pub cis_client: T,
    pub dinopark_settings: DinoParkServices,
}

fn field_error(msg: &str, e: impl std::fmt::Display) -> FieldError {
    let error = format!("{}: {}", msg, e);
    FieldError::new(msg, graphql_value!({ "internal_error": error }))
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
) -> FieldResult<Profile> {
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

    update
        .update_profile(
            &mut profile,
            cis_client.get_secret_store(),
            &dinopark_settings.fossil,
        )
        .map_err(|e| field_error("unable update/sign profle", e))?;
    let ret = test::block_on(cis_client.update_user(&user_id, profile))?;
    info!("update returned: {}", ret);
    let updated_profile = test::block_on(cis_client.get_user_by(&user_id, &GetBy::UserId, None))?;
    if dinopark_settings.lookout.internal_update_enabled {
        if let Err(e) = Client::new()
            .post(&dinopark_settings.lookout.internal_update_endpoint)
            .json(&updated_profile)
            .send()
        {
            error!("unable to post to lookout: {}", e);
        }
    }
    Ok(updated_profile)
}

#[juniper::object{
    Context = (UserId, Option<Scope>)
}]
impl<T: AsyncCisClientTrait> Query<T> {
    fn profile(username: Option<String>) -> FieldResult<Profile> {
        let executor = &executor;
        let (user_id, scope) = executor.context();
        let (id, by, filter) = if let Some(username) = username {
            (
                username,
                &GetBy::PrimaryUsername,
                scope
                    .as_ref()
                    .map(|s| s.scope.as_str())
                    .unwrap_or_else(|| "public"),
            )
        } else {
            (user_id.user_id.clone(), &GetBy::UserId, "private")
        };
        get_profile(id, &self.cis_client, by, filter)
    }
}

#[juniper::object{
    Context = (UserId, Option<Scope>)
}]
impl<T: AsyncCisClientTrait> Mutation<T> {
    fn profile(update: InputProfile) -> FieldResult<Profile> {
        let executor = &executor;
        update_profile(
            update,
            &self.cis_client,
            &self.dinopark_settings,
            &Some(executor.context().0.user_id.clone()),
        )
    }
}

pub type Schema<T> = RootNode<'static, Query<T>, Mutation<T>>;
