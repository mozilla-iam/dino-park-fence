use crate::graphql_api::root::{Mutation, Query, Schema};
use crate::permissions::Scope;
use crate::permissions::UserId;
use crate::settings::DinoParkServices;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::HttpResponse;
use actix_web::web::Json;
use actix_web::Result;
use actix_web::web::Data;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use cis_client::sync::client::CisClientTrait;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::sync::Arc;

#[derive(Serialize)]
pub struct ProfileByUsername {
    username: String,
}

#[derive(Clone)]
pub struct GraphQlState<T: CisClientTrait + 'static> {
    schema: Arc<Schema<T>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphQlData(GraphQLRequest);

fn graphiql(_: UserId) -> Result<HttpResponse> {
    let html = graphiql_source("/api/v4/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql<T: CisClientTrait>(
    body: Json<GraphQlData>,
    state: Data<GraphQlState<T>>,
    user_id: UserId,
    scope: Option<Scope>,
) -> Result<HttpResponse> {
    info!("graphql for {:?} → {:?}", user_id, scope);
    let graphql_data = body.0;
    let query_json = serde_json::to_value(&graphql_data.0)?;
    info!("body: {:#?}", query_json);
    let res = graphql_data.0.execute(&state.schema, &(user_id, scope));
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

pub fn graphql_app<T: CisClientTrait + Clone + Send + Sync + 'static>(
    cis_client: T,
    dinopark_settings: &DinoParkServices,
) -> impl HttpServiceFactory {
    let schema = Schema::new(
        Query {
            cis_client: cis_client.clone(),
            dinopark_settings: dinopark_settings.clone(),
        },
        Mutation {
            cis_client,
            dinopark_settings: dinopark_settings.clone(),
        },
    );

    web::scope("/graphql")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(GraphQlState { schema: Arc::new(schema) })
        .service(
            web::resource("")
                .route(web::post().to(graphql::<T>)),
        )
        .service(web::resource("/graphiql").route(web::get().to(graphiql)))
}
