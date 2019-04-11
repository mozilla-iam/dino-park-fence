use crate::settings::Orgchart;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::Json;
use actix_web::Path;
use actix_web::Result;
use actix_web::State;
use percent_encoding::utf8_percent_encode;
use percent_encoding::PATH_SEGMENT_ENCODE_SET;
use reqwest::get;
use serde_json::Value;

fn handle_full(state: State<Orgchart>) -> Result<Json<Value>> {
    let mut res = get(&state.full_endpoint).map_err(error::ErrorBadRequest)?;
    let json: Value = res.json().map_err(error::ErrorBadRequest)?;
    Ok(Json(json))
}

fn handle_trace(state: State<Orgchart>, username: Path<String>) -> Result<Json<Value>> {
    info!("getting {}", username);
    let safe_username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
    let mut res = get(&format!("{}{}", state.trace_endpoint, safe_username))
        .map_err(error::ErrorBadRequest)?;
    let json: Value = res.json().map_err(error::ErrorBadRequest)?;
    Ok(Json(json))
}

fn handle_related(state: State<Orgchart>, username: Path<String>) -> Result<Json<Value>> {
    info!("getting {}", username);
    let safe_username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
    let mut res = get(&format!("{}{}", state.related_endpoint, safe_username))
        .map_err(error::ErrorBadRequest)?;
    let json: Value = res.json().map_err(error::ErrorBadRequest)?;
    Ok(Json(json))
}

pub fn orgchart_app(settings: &Orgchart) -> App<Orgchart> {
    App::with_state(settings.clone())
        .prefix("/api/v4/orgchart")
        .configure(|app| {
            Cors::for_app(app)
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .resource("/related/{username}", move |r| {
                    r.method(http::Method::GET).with(handle_related)
                })
                .resource("/trace/{username}", move |r| {
                    r.method(http::Method::GET).with(handle_trace)
                })
                .resource("", move |r| r.method(http::Method::GET).with(handle_full))
                .register()
        })
}
