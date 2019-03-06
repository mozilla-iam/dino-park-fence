use crate::graphql_api::input::InputProfile;
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

pub struct Query<T: CisClientTrait + Clone> {
    pub cis_client: T,
}

fn field_error(msg: &str, e: impl std::fmt::Display) -> FieldError {
    let error = format!("{}: {}", msg, e);
    FieldError::new(msg, graphql_value!({ "internal_error": error }))
}

fn get_profile(id: String, cis_client: &impl CisClientTrait, by: &GetBy) -> FieldResult<Profile> {
    let profile = cis_client.get_user_by(&id, by, None)?;
    Ok(profile)
}

pub struct Mutation<T: CisClientTrait + Clone> {
    pub cis_client: T,
}

fn update_profile(
    update: InputProfile,
    cis_client: &impl CisClientTrait,
    user: &Option<String>,
) -> FieldResult<Profile> {
    let user_id = user
        .clone()
        .ok_or_else(|| field_error("no username in query or scopt", "?!"))?;
    let mut profile = cis_client.get_user_by(&user_id, &GetBy::UserId, None)?;
    update
        .update_profile(&mut profile, cis_client.get_secret_store())
        .map_err(|e| field_error("unable update/sign profle", e))?;
    let ret = cis_client.update_user(&user_id, profile)?;
    info!("update returned: {}", ret);
    let updated_profile = cis_client.get_user_by(&user_id, &GetBy::UserId, None)?;
    Ok(updated_profile)
}

// generated via graphql_object!
impl<T: CisClientTrait + Clone> GraphQLType<DefaultScalarValue> for Query<T> {
    type Context = Option<String>;
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
                    let (id, by) = if let Some(username) = username {
                        (username, &GetBy::PrimaryUsername)
                    } else if let Some(id) = executor.context() {
                        (id.clone(), &GetBy::UserId)
                    } else {
                        return Err(field_error("no username in query or scopt", "?!"));
                    };
                    get_profile(id, &self.cis_client, by)
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
    type Context = Option<String>;
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
                    update_profile(update, &self.cis_client, executor.context())
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
