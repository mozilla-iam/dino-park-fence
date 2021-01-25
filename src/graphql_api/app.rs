use crate::error::ApiError;
use crate::graphql_api::root::{Mutation, Query, Schema};
use crate::metrics::Metrics;
use crate::settings::DinoParkServices;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::web::Data;
use actix_web::HttpResponse;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use dino_park_guard::guard;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper_actix::graphql_handler;
use log::info;
use std::sync::Arc;

#[derive(Clone)]
pub struct GraphQlState<T: AsyncCisClientTrait + Send + Sync + 'static> {
    schema: Arc<Schema<T>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphQlData(GraphQLRequest);

#[guard(Staff)]
async fn graphiql() -> Result<HttpResponse, ApiError> {
    let html = graphiql_source("/api/v4/graphql", None);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[guard(Public)]
async fn graphql<T: AsyncCisClientTrait + Send + Sync>(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    state: Data<GraphQlState<T>>,
    scope_and_user: ScopeAndUser,
    metrics: Data<Metrics>,
) -> Result<HttpResponse, ApiError> {
    info!(
        "graphql for {:?} → {:?}",
        &scope_and_user.user_id, &scope_and_user.scope
    );
    let schema = Arc::clone(&state.schema);
    graphql_handler(&schema, &(scope_and_user, (*metrics).clone()), req, payload).await.map_err(|_| ApiError::Unknown)
}

pub fn graphql_app<T: AsyncCisClientTrait + Clone + Send + Sync + 'static>(
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
        juniper::EmptySubscription::default(),
    );

    web::scope("/graphql")
        .data(GraphQlState {
            schema: Arc::new(schema),
        })
        .data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("").route(web::post().to(graphql::<T>)))
        .service(web::resource("/graphiql").route(web::get().to(graphiql)))
}
