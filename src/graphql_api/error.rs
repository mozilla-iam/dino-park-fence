use juniper::FieldError;

pub fn field_error(msg: &str, e: impl std::fmt::Display) -> FieldError {
    let error = format!("{msg}: {e}");
    FieldError::new(msg, graphql_value!({ "internal_error": error }))
}
