use crate::settings::Orgchart;
use actix_web::client::Client;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::web;
use actix_web::web::BytesMut;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Result;
use futures::Future;
use futures::Stream;
use percent_encoding::utf8_percent_encode;
use percent_encoding::PATH_SEGMENT_ENCODE_SET;
use reqwest::get;
use serde_json::Value;

fn handle_full(
    state: Data<Orgchart>,
    client: Data<Client>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    client
        .get(&state.full_endpoint)
        .send()
        .map_err(Error::from) // <- convert SendRequestError to an Error
        .and_then(|resp| {
            resp.from_err()
                .fold(BytesMut::new(), |mut acc, chunk| {
                    acc.extend_from_slice(&chunk);
                    Ok::<_, Error>(acc)
                })
                .and_then(|body| {
                    serde_json::from_slice::<Value>(&body).map_err(error::ErrorBadRequest)
                })
                .map(|o| HttpResponse::Ok().json(o))
        })
        .map_err(error::ErrorBadRequest)
}

fn handle_trace(state: Data<Orgchart>, username: Path<String>) -> Result<Json<Value>> {
    let safe_username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
    let mut res = get(&format!("{}{}", state.trace_endpoint, safe_username))
        .map_err(error::ErrorBadRequest)?;
    let json: Value = res.json().map_err(error::ErrorBadRequest)?;
    Ok(Json(json))
}

fn handle_related(state: Data<Orgchart>, username: Path<String>) -> Result<Json<Value>> {
    let safe_username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
    let mut res = get(&format!("{}{}", state.related_endpoint, safe_username))
        .map_err(error::ErrorBadRequest)?;
    let json: Value = res.json().map_err(error::ErrorBadRequest)?;
    Ok(Json(json))
}

pub fn orgchart_app(settings: &Orgchart) -> impl HttpServiceFactory {
    let client = Client::default();
    web::scope("/orgchart")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(settings.clone())
        .data(client)
        .service(web::resource("").route(web::get().to_async(handle_full)))
        .service(web::resource("/related/{username}").route(web::get().to(handle_related)))
        .service(web::resource("/trace/{username}").route(web::get().to(handle_trace)))
}
