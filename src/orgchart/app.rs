use crate::proxy::proxy;
use crate::settings::Orgchart;
use actix_cors::Cors;
use actix_web::client::Client;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::Error;
use actix_web::HttpResponse;
use dino_park_gate::scope::ScopeAndUser;
use futures::future::Either;
use futures::Future;
use futures::IntoFuture;
use percent_encoding::utf8_percent_encode;
use percent_encoding::AsciiSet;
use percent_encoding::CONTROLS;

pub const USERINFO_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|');

fn handle_full(
    client: Data<Client>,
    state: Data<Orgchart>,
    scope_and_user: ScopeAndUser,
) -> impl Future<Item = HttpResponse, Error = Error> {
    if scope_and_user.scope == "staff" {
        Either::A(proxy(&*client, &state.full_endpoint))
    } else {
        Either::B(Err::<HttpResponse, _>(error::ErrorForbidden("not staff")).into_future())
    }
}

fn handle_trace(
    client: Data<Client>,
    state: Data<Orgchart>,
    scope_and_user: ScopeAndUser,
    username: Path<String>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let safe_username = utf8_percent_encode(&username, USERINFO_ENCODE_SET);
    if scope_and_user.scope == "staff" {
        Either::A(proxy(
            &*client,
            &format!("{}{}", state.trace_endpoint, safe_username),
        ))
    } else {
        Either::B(Err::<HttpResponse, _>(error::ErrorForbidden("not staff")).into_future())
    }
}

fn handle_related(
    client: Data<Client>,
    state: Data<Orgchart>,
    scope_and_user: ScopeAndUser,
    username: Path<String>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let safe_username = utf8_percent_encode(&username, USERINFO_ENCODE_SET);
    if scope_and_user.scope == "staff" {
        Either::A(proxy(
            &*client,
            &format!("{}{}", state.related_endpoint, safe_username),
        ))
    } else {
        Either::B(Err::<HttpResponse, _>(error::ErrorForbidden("not staff")).into_future())
    }
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
