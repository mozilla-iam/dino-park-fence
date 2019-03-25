use crate::settings::Search;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::Json;
use actix_web::Query;
use actix_web::Result;
use actix_web::State;
use reqwest::Client;
use serde_json::Value;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    w: String,
}

fn handle_simple(state: State<Search>, query: Query<SearchQuery>) -> Result<Json<Value>> {
    let mut res = Client::new()
        .get(&format!("{}staff/", state.simple_endpoint))
        .query(&[("q", &query.q), ("w", &query.w)])
        .send()
        .map_err(error::ErrorBadRequest)?;
    let json: Value = res.json().map_err(error::ErrorBadRequest)?;
    Ok(Json(json))
}

pub fn search_app(settings: &Search) -> App<Search> {
    App::with_state(settings.clone())
        .prefix("/api/v4/search")
        .configure(|app| {
            Cors::for_app(app)
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .resource("/simple/", move |r| {
                    r.method(http::Method::GET).with(handle_simple)
                })
                .register()
        })
}
