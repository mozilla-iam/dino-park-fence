use crate::settings::Orgchart;
use actix_web::dev::Handler;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::Json;
use actix_web::Path;
use actix_web::Result;
use reqwest::get;
use serde_json::Value;

struct FullHandler {
    full_endpoint: String,
}
impl FullHandler {
    pub fn new(full_endpoint: &str) -> Self {
        FullHandler {
            full_endpoint: full_endpoint.to_owned(),
        }
    }
}

impl<S> Handler<S> for FullHandler {
    type Result = Result<Json<Value>>;

    fn handle(&self, _: &HttpRequest<S>) -> Self::Result {
        let mut res = get(&self.full_endpoint).map_err(error::ErrorBadRequest)?;
        let json: Value = res.json().map_err(error::ErrorBadRequest)?;
        Ok(Json(json))
    }
}

struct TraceHandler {
    trace_endpoint: String,
}
impl TraceHandler {
    pub fn new(trace_endpoint: &str) -> Self {
        TraceHandler {
            trace_endpoint: trace_endpoint.to_owned(),
        }
    }
}

impl<S> Handler<S> for TraceHandler {
    type Result = Result<Json<Value>>;

    fn handle(&self, req: &HttpRequest<S>) -> Self::Result {
        let username = Path::<String>::extract(req)?;
        info!("getting {}", username);
        let mut res =
            get(&format!("{}{}", self.trace_endpoint, username)).map_err(error::ErrorBadRequest)?;
        let json: Value = res.json().map_err(error::ErrorBadRequest)?;
        Ok(Json(json))
    }
}

struct RelatedHandler {
    related_endpoint: String,
}
impl RelatedHandler {
    pub fn new(related_endpoint: &str) -> Self {
        RelatedHandler {
            related_endpoint: related_endpoint.to_owned(),
        }
    }
}

impl<S> Handler<S> for RelatedHandler {
    type Result = Result<Json<Value>>;

    fn handle(&self, req: &HttpRequest<S>) -> Self::Result {
        let username = Path::<String>::extract(req)?;
        info!("getting {}", username);
        let mut res = get(&format!("{}{}", self.related_endpoint, username))
            .map_err(error::ErrorBadRequest)?;
        let json: Value = res.json().map_err(error::ErrorBadRequest)?;
        Ok(Json(json))
    }
}

pub fn orgchart_app(settings: &Orgchart) -> App {
    App::new().prefix("/api/v4/orgchart").configure(|app| {
        let related_handler = RelatedHandler::new(&settings.related_endpoint);
        let trace_handler = TraceHandler::new(&settings.trace_endpoint);
        let full_handler = FullHandler::new(&settings.full_endpoint);
        Cors::for_app(app)
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/related/{username}", move |r| r.h(related_handler))
            .resource("/trace/{username}", move |r| r.h(trace_handler))
            .resource("", move |r| r.h(full_handler))
            .register()
    })
}
