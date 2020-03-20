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
use dino_park_trust::Trust;
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

async fn handle_full(
    client: Data<Client>,
    state: Data<Orgchart>,
    scope_and_user: ScopeAndUser,
) -> Result<HttpResponse, Error> {
    if scope_and_user.scope == Trust::Staff {
        proxy(&*client, &state.full_endpoint).await
    } else {
        Err::<HttpResponse, _>(error::ErrorForbidden("not staff"))
    }
}

async fn handle_trace(
    client: Data<Client>,
    state: Data<Orgchart>,
    scope_and_user: ScopeAndUser,
    username: Path<String>,
) -> Result<HttpResponse, Error> {
    let safe_username = utf8_percent_encode(&username, USERINFO_ENCODE_SET);
    if scope_and_user.scope == Trust::Staff {
        proxy(
            &*client,
            &format!("{}{}", state.trace_endpoint, safe_username),
        )
        .await
    } else {
        Err::<HttpResponse, _>(error::ErrorForbidden("not staff"))
    }
}

async fn handle_related(
    client: Data<Client>,
    state: Data<Orgchart>,
    scope_and_user: ScopeAndUser,
    username: Path<String>,
) -> Result<HttpResponse, Error> {
    let safe_username = utf8_percent_encode(&username, USERINFO_ENCODE_SET);
    if scope_and_user.scope == Trust::Staff {
        proxy(
            &*client,
            &format!("{}{}", state.related_endpoint, safe_username),
        )
        .await
    } else {
        Err::<HttpResponse, _>(error::ErrorForbidden("not staff"))
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
                .max_age(3600)
                .finish(),
        )
        .data(settings.clone())
        .data(client)
        .service(web::resource("").route(web::get().to(handle_full)))
        .service(web::resource("/related/{username}").route(web::get().to(handle_related)))
        .service(web::resource("/trace/{username}").route(web::get().to(handle_trace)))
}
