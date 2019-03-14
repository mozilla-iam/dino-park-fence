use crate::settings::Search;
use actix_web::dev::Handler;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::Json;
use actix_web::Result;
use reqwest::Client;
use serde_json::Value;

struct SimpleHandler {
    simple_endpoint: String,
}
impl SimpleHandler {
    pub fn new(simple_endpoint: &str) -> Self {
        SimpleHandler {
            simple_endpoint: simple_endpoint.to_owned(),
        }
    }
}

impl<S> Handler<S> for SimpleHandler {
    type Result = Result<Json<Value>>;

    fn handle(&self, req: &HttpRequest<S>) -> Self::Result {
        let query_map = req.query();
        let query: Vec<(&String, &String)> = query_map.iter().map(|(k, v)| (k, v)).collect();
        let mut res = Client::new()
            .get(&format!("{}staff/", self.simple_endpoint))
            .query(&query)
            .send()
            .map_err(error::ErrorBadRequest)?;
        let json: Value = res.json().map_err(error::ErrorBadRequest)?;
        Ok(Json(json))
    }
}

pub fn search_app(settings: &Search) -> App {
    App::new().prefix("/api/v4/search").configure(|app| {
        let simple_handler = SimpleHandler::new(&settings.simple_endpoint);
        Cors::for_app(app)
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/simple/", move |r| r.h(simple_handler))
            .register()
    })
}
