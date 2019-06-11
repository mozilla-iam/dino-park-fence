use crate::graphql_api::root::{Mutation, Query, Schema};
use crate::settings::DinoParkServices;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Result;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use futures::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::sync::Arc;

#[derive(Clone)]
pub struct GraphQlState<T: AsyncCisClientTrait + 'static> {
    schema: Arc<Schema<T>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphQlData(GraphQLRequest);

fn graphiql() -> Result<HttpResponse> {
    let html = graphiql_source("/api/v4/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql<T: AsyncCisClientTrait + Send + Sync>(
    data: Json<GraphQLRequest>,
    state: Data<GraphQlState<T>>,
    scope_and_user: ScopeAndUser,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    info!(
        "graphql for {:?} → {:?}",
        &scope_and_user.user_id, &scope_and_user.scope
    );
    let schema = Arc::clone(&state.schema);
    let res = web::block(move || {
        let r = data.execute(&schema, &(scope_and_user));
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&r)?)
    })
    .map_err(Error::from);
    Box::new(res.and_then(|res| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(res))
    }))
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
    );

    web::scope("/graphql")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(GraphQlState {
            schema: Arc::new(schema),
        })
        .data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("").route(web::post().to(graphql::<T>)))
        .service(web::resource("/graphiql").route(web::get().to(graphiql)))
}
