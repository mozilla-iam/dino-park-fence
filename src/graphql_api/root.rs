use crate::graphql_api::input::InputProfile;
use crate::permissions::Scope;
use crate::permissions::UserId;
use crate::settings::DinoParkServices;
use cis_client::client::CisClientTrait;
use cis_client::client::GetBy;
use cis_profile::schema::Profile;
use juniper::meta::MetaType;
use juniper::Arguments;
use juniper::DefaultScalarValue;
use juniper::ExecutionResult;
use juniper::Executor;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::GraphQLType;
use juniper::IntoResolvable;
use juniper::Registry;
use juniper::RootNode;
use juniper::ScalarRefValue;
use juniper::Value;
use reqwest::Client;

pub struct Query<T: CisClientTrait + Clone> {
    pub cis_client: T,
    pub dinopark_settings: DinoParkServices,
}

fn field_error(msg: &str, e: impl std::fmt::Display) -> FieldError {
    let error = format!("{}: {}", msg, e);
    FieldError::new(msg, graphql_value!({ "internal_error": error }))
}

fn get_profile(
    id: String,
    cis_client: &impl CisClientTrait,
    by: &GetBy,
    filter: &str,
) -> FieldResult<Profile> {
    let profile = cis_client.get_user_by(&id, by, Some(&filter))?;
    if profile.active.value.unwrap_or_default()
        || match by {
            GetBy::UserId => true,
            _ => false,
        }
    {
        Ok(profile)
    } else {
        Err(field_error(
            "unknown_profile",
            "Profile not available in CIS.",
        ))
    }
}

pub struct Mutation<T: CisClientTrait + Clone> {
    pub cis_client: T,
    pub dinopark_settings: DinoParkServices,
}

fn update_profile(
    update: InputProfile,
    cis_client: &impl CisClientTrait,
    dinopark_settings: &DinoParkServices,
    user: &Option<String>,
) -> FieldResult<Profile> {
    let user_id = user
        .clone()
        .ok_or_else(|| field_error("no username in query or scope", "?!"))?;
    let mut profile = cis_client.get_user_by(&user_id, &GetBy::UserId, None)?;
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
            if cis_client
                .get_user_by(updated_username, &GetBy::PrimaryUsername, None)
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
    let ret = cis_client.update_user(&user_id, profile)?;
    info!("update returned: {}", ret);
    let updated_profile = cis_client.get_user_by(&user_id, &GetBy::UserId, None)?;
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

// generated via graphql_object!
impl<T: CisClientTrait + Clone> GraphQLType<DefaultScalarValue> for Query<T> {
    type Context = (UserId, Option<Scope>);
    type TypeInfo = ();
    fn name(_: &Self::TypeInfo) -> Option<&str> {
        Some("Query")
    }
    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, DefaultScalarValue>,
    ) -> MetaType<'r, DefaultScalarValue>
    where
        for<'__b> &'__b DefaultScalarValue: ScalarRefValue<'__b>,
        DefaultScalarValue: 'r,
    {
        let fields = &[
            registry
                .field_convert::<&str, _, Self::Context>("apiVersion", info)
                .push_docstring(&[]),
            registry
                .field_convert::<FieldResult<Profile>, _, Self::Context>("profile", info)
                .push_docstring(&[])
                .argument(
                    registry
                        .arg::<Option<String>>("username", info)
                        .push_docstring(&[]),
                ),
        ];
        registry
            .build_object_type::<Query<T>>(info, fields)
            .into_meta()
    }

    fn concrete_type_name(&self, _: &Self::Context, _: &Self::TypeInfo) -> String {
        "Query".to_owned()
    }

    fn resolve_field(
        &self,
        _: &Self::TypeInfo,
        field: &str,
        args: &Arguments<DefaultScalarValue>,
        executor: &Executor<Self::Context, DefaultScalarValue>,
    ) -> ExecutionResult<DefaultScalarValue> {
        if field == "apiVersion" {
            let result: &str = "1.0";
            return IntoResolvable::into(result, executor.context()).and_then(|res| match res {
                Some((ctx, r)) => executor.replaced_context(ctx).resolve_with_ctx(&(), &r),
                None => Ok(Value::null()),
            });
        }
        if field == "profile" {
            let result: FieldResult<Profile> = {
                let username: Option<String> = args
                    .get("username")
                    .expect("Argument username missing - validation must have failed");
                let executor = &executor;
                {
                    let (user_id, _scope) = executor.context();
                    let (id, by, filter) = if let Some(username) = username {
                        (username, &GetBy::PrimaryUsername, "staff")
                    } else {
                        (user_id.user_id.clone(), &GetBy::UserId, "private")
                    };
                    get_profile(id, &self.cis_client, by, filter)
                }
            };
            return IntoResolvable::into(result, executor.context()).and_then(|res| match res {
                Some((ctx, r)) => executor.replaced_context(ctx).resolve_with_ctx(&(), &r),
                None => Ok(Value::null()),
            });
        }
        Err(field_error(
            "error",
            format!("Field {} not found on type Query", field),
        ))
    }
}

// generated via graphql_object!
impl<T: CisClientTrait + Clone> GraphQLType<DefaultScalarValue> for Mutation<T> {
    type Context = (UserId, Option<Scope>);
    type TypeInfo = ();
    fn name(_: &Self::TypeInfo) -> Option<&str> {
        Some("Mutation")
    }
    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut Registry<'r, DefaultScalarValue>,
    ) -> MetaType<'r, DefaultScalarValue>
    where
        for<'__b> &'__b DefaultScalarValue: ScalarRefValue<'__b>,
        DefaultScalarValue: 'r,
    {
        let fields = &[
            registry
                .field_convert::<&str, _, Self::Context>("apiVersion", info)
                .push_docstring(&[]),
            registry
                .field_convert::<FieldResult<Profile>, _, Self::Context>("profile", info)
                .push_docstring(&[])
                .argument(
                    registry
                        .arg::<InputProfile>("update", info)
                        .push_docstring(&[]),
                ),
        ];
        registry
            .build_object_type::<Mutation<T>>(info, fields)
            .into_meta()
    }

    fn concrete_type_name(&self, _: &Self::Context, _: &Self::TypeInfo) -> String {
        "Mutation".to_owned()
    }

    fn resolve_field(
        &self,
        _: &Self::TypeInfo,
        field: &str,
        args: &Arguments<DefaultScalarValue>,
        executor: &Executor<Self::Context, DefaultScalarValue>,
    ) -> ExecutionResult<DefaultScalarValue> {
        if field == "apiVersion" {
            let result: &str = "1.0";
            return IntoResolvable::into(result, executor.context()).and_then(|res| match res {
                Some((ctx, r)) => executor.replaced_context(ctx).resolve_with_ctx(&(), &r),
                None => Ok(Value::null()),
            });
        }
        if field == "profile" {
            let result: FieldResult<Profile> = {
                let update: InputProfile = args
                    .get("update")
                    .expect("Argument update missing - validation must have failed");
                let executor = &executor;
                {
                    update_profile(
                        update,
                        &self.cis_client,
                        &self.dinopark_settings,
                        &Some(executor.context().0.user_id.clone()),
                    )
                }
            };
            return IntoResolvable::into(result, executor.context()).and_then(|res| match res {
                Some((ctx, r)) => executor.replaced_context(ctx).resolve_with_ctx(&(), &r),
                None => Ok(Value::null()),
            });
        }
        Err(field_error(
            "error",
            format!("Field {} not found on type Mutation", field),
        ))
    }
}

pub type Schema<T> = RootNode<'static, Query<T>, Mutation<T>>;
