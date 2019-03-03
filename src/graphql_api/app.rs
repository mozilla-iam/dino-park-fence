use std::sync::Arc;

use actix::prelude::*;
use actix_web::middleware::cors::Cors;
use actix_web::{
    http, App, AsyncResponder, Error, FutureResponse, HttpMessage, HttpRequest, HttpResponse,
};
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use crate::cis::client::CisClientTrait;
use crate::graphql_api::root::{Mutation, Query, Schema};

pub struct AppState<T: CisClientTrait + Clone + 'static> {
    executor: Addr<GraphQLExecutor<T>>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLData {
    data: GraphQLRequest,
    user: Option<String>,
}

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor<T: CisClientTrait + Clone> {
    schema: Arc<Schema<T>>,
}

impl<T: CisClientTrait + Clone> GraphQLExecutor<T> {
    fn new(schema: Arc<Schema<T>>) -> GraphQLExecutor<T> {
        GraphQLExecutor { schema }
    }
}

impl<T: CisClientTrait + Clone + 'static> Actor for GraphQLExecutor<T> {
    type Context = SyncContext<Self>;
}

impl<T: CisClientTrait + Clone + 'static> Handler<GraphQLData> for GraphQLExecutor<T> {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg.data.execute(&self.schema, &msg.user);
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

fn graphiql<T: CisClientTrait + Clone>(
    _req: &HttpRequest<AppState<T>>,
) -> Result<HttpResponse, Error> {
    let html = graphiql_source("/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql<T: CisClientTrait + Clone + 'static>(
    req: HttpRequest<AppState<T>>,
) -> FutureResponse<HttpResponse> {
    let headers = req.headers();
    let user = headers
        .get("x-forwarded-user")
        .and_then(|v| match v.to_str() {
            Ok(s) if s.contains("+hknall") => Some("hknall@mozilla.com"),
            Ok(s) => Some(s),
            Err(e) => {
                warn!("unable to decode user: {}", e);
                None
            }
        })
        .map(String::from);
    req.json::<GraphQLRequest>()
        .from_err()
        .and_then(move |q| {
            req.state()
                .executor
                .send(GraphQLData { data: q, user })
                .from_err()
                .and_then(|res| match res {
                    Ok(user) => Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(user)),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

pub fn graphql_app<T: CisClientTrait + Clone + Send + Sync + 'static>(
    cis_client: T,
) -> App<AppState<T>> {
    let schema = Arc::new(Schema::new(
        Query {
            cis_client: cis_client.clone(),
        },
        Mutation { cis_client },
    ));
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
