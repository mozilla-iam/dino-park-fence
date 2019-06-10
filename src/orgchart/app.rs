use crate::proxy::proxy;
use crate::settings::Orgchart;
use actix_web::client::Client;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::Error;
use actix_web::HttpResponse;
use futures::Future;
use percent_encoding::utf8_percent_encode;
use percent_encoding::PATH_SEGMENT_ENCODE_SET;

fn handle_full(
    client: Data<Client>,
    state: Data<Orgchart>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    proxy(&*client, &state.full_endpoint)
}

fn handle_trace(
    client: Data<Client>,
    state: Data<Orgchart>,
    username: Path<String>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let safe_username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
    proxy(
        &*client,
        &format!("{}{}", state.trace_endpoint, safe_username),
    )
}

fn handle_related(
    client: Data<Client>,
    state: Data<Orgchart>,
    username: Path<String>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let safe_username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
    proxy(
        &*client,
        &format!("{}{}", state.related_endpoint, safe_username),
    )
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
        .service(web::resource("/related/{username}").route(web::get().to_async(handle_related)))
        .service(web::resource("/trace/{username}").route(web::get().to_async(handle_trace)))
}
