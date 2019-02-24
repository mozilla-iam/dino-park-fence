use std::sync::Arc;

use actix::prelude::*;
use actix_web::middleware::cors::Cors;
use actix_web::{
    http, App, AsyncResponder, Error, FutureResponse, HttpRequest, HttpResponse, Json, State,
};
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use crate::cis::config::Config;
use crate::graphql_api::root::{Mutation, Query, Schema};

pub struct AppState {
    executor: Addr<GraphQLExecutor>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
    schema: Arc<Schema>,
}

impl GraphQLExecutor {
    fn new(schema: Arc<Schema>) -> GraphQLExecutor {
        GraphQLExecutor { schema }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg.0.execute(&self.schema, &());
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let html = graphiql_source("http://127.0.0.1:8080/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql((st, data): (State<AppState>, Json<GraphQLData>)) -> FutureResponse<HttpResponse> {
    st.executor
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

pub fn graphql_app(cfg: Config) -> App<AppState> {
    let schema = Arc::new(Schema::new(Query { cfg: cfg.clone() }, Mutation { cfg }));
    let addr = SyncArbiter::start(3, move || GraphQLExecutor::new(schema.clone()));

    App::with_state(AppState {
        executor: addr.clone(),
    })
    .configure(|app| {
        Cors::for_app(app)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
            .resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))
            .register()
    })
}
