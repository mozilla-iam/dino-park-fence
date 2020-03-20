use crate::graphql_api::root::{Mutation, Query, Schema};
use crate::metrics::Metrics;
use crate::settings::DinoParkServices;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Result;
use cis_client::sync::client::CisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use log::info;
use std::sync::Arc;

#[derive(Clone)]
pub struct GraphQlState<T: CisClientTrait + 'static> {
    schema: Arc<Schema<T>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphQlData(GraphQLRequest);

async fn graphiql() -> Result<HttpResponse> {
    let html = graphiql_source("/api/v4/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn graphql<T: CisClientTrait + Send + Sync>(
    data: Json<GraphQLRequest>,
    state: Data<GraphQlState<T>>,
    scope_and_user: ScopeAndUser,
    metrics: Data<Metrics>,
) -> Result<HttpResponse, Error> {
    info!(
        "graphql for {:?} → {:?}",
        &scope_and_user.user_id, &scope_and_user.scope
    );
    let schema = Arc::clone(&state.schema);
    let res = web::block(move || {
        let r = data.execute(&schema, &(scope_and_user, (*metrics).clone()));
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&r)?)
    })
    .await
    .map_err(Error::from)?;
    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(res))
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
                .max_age(3600)
                .finish(),
        )
        .data(GraphQlState {
            schema: Arc::new(schema),
        })
        .data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("").route(web::post().to(graphql::<T>)))
        .service(web::resource("/graphiql").route(web::get().to(graphiql)))
}
