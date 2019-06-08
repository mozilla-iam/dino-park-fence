use crate::settings::Search;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::web::Query;
use actix_web::Result;
use reqwest::Client;
use serde_json::Value;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    w: String,
    a: Option<String>,
}

fn handle_simple(search: Data<Search>, query: Query<SearchQuery>) -> Result<Json<Value>> {
    let mut client = Client::new()
        .get(&format!("{}staff/", search.simple_endpoint))
        .query(&[("q", &query.q), ("w", &query.w)]);
    if let Some(a) = &query.a {
        client = client.query(&[("a", a)]);
    }
    let mut res = client.send().map_err(error::ErrorBadRequest)?;
    let json: Value = res.json().map_err(error::ErrorBadRequest)?;
    Ok(Json(json))
}

pub fn search_app(settings: &Search) -> impl HttpServiceFactory {
    web::scope("/search")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(settings.clone())
        .service(web::resource("/simple").route(web::get().to(handle_simple)))
}
