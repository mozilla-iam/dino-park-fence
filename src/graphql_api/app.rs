use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::Result;
use actix_web::State;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::sync::Arc;

use crate::graphql_api::root::{Mutation, Query, Schema};
use crate::permissions::Scope;
use crate::permissions::UserId;
use cis_client::client::CisClientTrait;

#[derive(Clone)]
pub struct GraphQlState<T: CisClientTrait + Clone + 'static> {
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

fn graphql<T: CisClientTrait + Clone>(
    body: Json<GraphQlData>,
    state: State<GraphQlState<T>>,
    user_id: UserId,
    scope: Option<Scope>,
) -> Result<HttpResponse> {
    info!("graphql for {:?} → {:?}", user_id, scope);
    let graphql_data = body.0;
    let res = graphql_data
        .0
        .execute(&state.schema, &Some(user_id.user_id));
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&res)?))
}

pub fn graphql_app<T: CisClientTrait + Clone + Send + Sync + 'static>(
    cis_client: T,
) -> App<GraphQlState<T>> {
    let schema = Schema::new(
        Query {
            cis_client: cis_client.clone(),
        },
        Mutation { cis_client },
    );

    App::with_state(GraphQlState {
        schema: Arc::new(schema),
    })
    .prefix("/api/v4/graphql")
    .configure(|app| {
        Cors::for_app(app)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("", |r| r.method(http::Method::POST).with(graphql))
            .resource("/graphiql", |r| r.method(http::Method::GET).with(graphiql))
            .register()
    })
}
