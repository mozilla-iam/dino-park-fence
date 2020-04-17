use crate::error::ApiError;
use crate::proxy::proxy;
use crate::settings::Search;
use actix_web::client::Client;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Query;
use actix_web::HttpResponse;
use dino_park_gate::scope::ScopeAndUser;
use dino_park_guard::guard;
use url::Url;

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    w: String,
    a: Option<String>,
}

#[guard(Public)]
async fn handle_simple(
    client: Data<Client>,
    search: Data<Search>,
    scope_and_user: ScopeAndUser,
    query: Query<SearchQuery>,
) -> Result<HttpResponse, ApiError> {
    let mut url = Url::parse(&search.simple_endpoint).map_err(|_| ApiError::Unknown)?;
    url.path_segments_mut()
        .map_err(|_| ApiError::Unknown)?
        .pop_if_empty()
        .push(&scope_and_user.scope.as_str())
        .push("");
    url.query_pairs_mut()
        .append_pair("q", &query.q)
        .append_pair("w", &query.w);
    if let Some(a) = &query.a {
        url.query_pairs_mut().append_pair("a", &a);
    }
    proxy(&*client, url.as_str()).await
}

pub fn search_app(settings: &Search) -> impl HttpServiceFactory {
    let client = Client::default();
    web::scope("/search")
        .data(client)
        .data(settings.clone())
        .service(web::resource("/simple/").route(web::get().to(handle_simple)))
}
