use juniper::graphql_object;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;

use cis_profile::schema::Profile;

use crate::cis::client::CisClient;
use crate::cis::client::GetBy;
use crate::graphql_api::input::InputProfile;

pub struct Query {
    pub cis_client: CisClient,
}

fn field_error(msg: &str, e: impl std::fmt::Display) -> FieldError {
    let error = format!("{}: {}", msg, e);
    FieldError::new(msg, graphql_value!({ "internal_error": error }))
}

fn get_profile(id: String, cis_client: &CisClient, by: &GetBy) -> FieldResult<Profile> {
    let profile = cis_client.get_user_by(&id, by, None)?;
    Ok(profile)
}

graphql_object!(Query: Option<String> |&self| {
    field apiVersion() -> &str {
        "1.0"
    }
    field profile(&executor, username: Option<String>) -> FieldResult<Profile> {
        let (id, by) = if let Some(username) = username {
            (username, &GetBy::PrimaryUsername)
        } else if let Some(id) = executor.context() {
            (id.clone(), &GetBy::PrimaryEmail)
        } else {
            (String::from("hknall@mozilla.com"), &GetBy::PrimaryEmail)
            //return Err(field_error("no username in query or scopt", "?!"));
        };
        get_profile(id, &self.cis_client, by)
    }
});

pub struct Mutation {
    pub cis_client: CisClient,
}

fn update_profile(
    update: InputProfile,
    cis_client: &CisClient,
    user: &Option<String>,
) -> FieldResult<Profile> {
    let user_id = user
        .clone()
        .unwrap_or_else(|| String::from("hknall@mozilla.com"));
    let mut profile = cis_client.get_user_by(&user_id, &GetBy::PrimaryEmail, None)?;
    update
        .update_profile(&mut profile, &cis_client.get_secret_store())
        .map_err(|e| field_error("unable update/sign profle", e))?;
    let ret = cis_client.update_user(profile)?;
    info!("update returned: {}", ret);
    let updated_profile = cis_client.get_user_by(&user_id, &GetBy::PrimaryEmail, None)?;
    Ok(updated_profile)
}

graphql_object!(Mutation: Option<String> |&self| {
    field apiVersion() -> &str {
        "1.0"
    }
    field profile(&executor, update: InputProfile,) -> FieldResult<Profile> {
        update_profile(update, &self.cis_client, executor.context())
    }
});

pub type Schema = RootNode<'static, Query, Mutation>;
