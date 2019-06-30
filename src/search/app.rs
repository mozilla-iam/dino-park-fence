use crate::proxy::proxy;
use crate::settings::Search;
use actix_cors::Cors;
use actix_web::client::Client;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Query;
use actix_web::Error;
use actix_web::HttpResponse;
use dino_park_gate::scope::ScopeAndUser;
use futures::Future;
use futures::IntoFuture;
use url::ParseError;
use url::Url;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    w: String,
    a: Option<String>,
}

fn handle_simple(
    client: Data<Client>,
    search: Data<Search>,
    scope_and_user: ScopeAndUser,
    query: Query<SearchQuery>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let url = Url::parse(&search.simple_endpoint);
    url.and_then(|mut url| {
        url.path_segments_mut()
            .map_err(|_| ParseError::RelativeUrlWithCannotBeABaseBase)?
            .pop_if_empty()
            .push(&scope_and_user.scope)
            .push("");
        url.query_pairs_mut()
            .append_pair("q", &query.q)
            .append_pair("w", &query.w);
        if let Some(a) = &query.a {
            url.query_pairs_mut().append_pair("a", &a);
        }
        Ok(url)
    })
    .into_future()
    .map_err(|e| error::UrlGenerationError::ParseError(e).into())
    .and_then(move |url| proxy(&*client, url.as_str()))
}

pub fn search_app(settings: &Search) -> impl HttpServiceFactory {
    let client = Client::default();
    web::scope("/search")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(client)
        .data(settings.clone())
        .service(web::resource("/simple/").route(web::get().to_async(handle_simple)))
}
