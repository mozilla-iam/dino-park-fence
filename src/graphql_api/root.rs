use juniper::graphql_object;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;

use cis_profile::schema::Profile;

use crate::cis::config::Config;
use crate::cis::person_api::get_user;
use crate::cis::person_api::update_user;
use crate::cis::person_api::GetBy;
use crate::graphql_api::input::InputProfile;

pub struct Query {
    pub cfg: Config,
}

fn field_error(msg: &str, e: impl std::fmt::Display) -> FieldError {
    let error = format!("{}: {}", msg, e);
    FieldError::new(msg, graphql_value!({ "internal_error": error }))
}

fn get_profile(username: Option<String>, cfg: &Config) -> FieldResult<Profile> {
    let username = username.unwrap_or_else(|| String::from("fiji"));
    let b = cfg
        .cis_client
        .bearer_store
        .get()
        .map_err(|e| field_error("unable to get token", e))?;
    let b1 = b
        .read()
        .map_err(|e| field_error("unable to read token", e))?;
    let token = &*b1.baerer_token_str;
    let profile = get_user(token, &username, &GetBy::PrimaryUsername, None)
        .map_err(|e| field_error("unable to get profile", e))?;
    Ok(profile)
}

graphql_object!(Query: () |&self| {

    field apiVersion() -> &str {
        "1.0"
    }

    field profile(&executor, username: Option<String>) -> FieldResult<Profile> {
        get_profile(username, &self.cfg)
    }
});

pub struct Mutation {
    pub cfg: Config,
}

fn update_profile(update: InputProfile, cfg: &Config) -> FieldResult<Profile> {
    let b = cfg
        .cis_client
        .bearer_store
        .get()
        .map_err(|e| field_error("unable to get token", e))?;
    let b1 = b
        .read()
        .map_err(|e| field_error("unable to read token", e))?;
    let token = &*b1.baerer_token_str;

    let user_id = "ad|Mozilla-LDAP|FMerz";
    let mut profile = get_user(&token, user_id, &GetBy::UserId, None)
        .map_err(|e| field_error("unable to get profle", e))?;
    update
        .update_profile(&mut profile, &cfg.secret_store)
        .map_err(|e| field_error("unable update/sign profle", e))?;
    let ret = update_user(&token, false, profile, &cfg.secret_store)
        .map_err(|e| field_error("unable to get profle", e))?;
    info!("update returned: {}", ret);
    let updated_profile = get_user(&token, user_id, &GetBy::UserId, None)
        .map_err(|e| field_error("unable to get updated profle", e))?;
    Ok(updated_profile)
}

graphql_object!(Mutation: () |&self| {

    field apiVersion() -> &str {
        "1.0"
    }

    field profile(&executor, update: InputProfile) -> FieldResult<Profile> {
        update_profile(update, &self.cfg)
    }
});

pub type Schema = RootNode<'static, Query, Mutation>;
