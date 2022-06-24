use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use cookie::Cookie;
use cookie::SameSite;

const KEEP_LOGGED_IN_COOKIE_NAME: &str = "pmo-kli";
const LOGIN_PATH: &str = "/";
const LOGOUT_PATH: &str = "/oauth/logout?redirect=/";

const FIVE_YEARS_IN_SECS: cookie::time::Duration = cookie::time::Duration::new(5 * 365 * 24 * 60 * 60, 0);

enum KeepLoggedIn {
    No,
    Yes,
}

impl From<KeepLoggedIn> for &str {
    fn from(o: KeepLoggedIn) -> Self {
        match o {
            KeepLoggedIn::No => "0",
            KeepLoggedIn::Yes => "1",
        }
    }
}

fn set_cookie_and_redirect(
    name: &'static str,
    value: &'static str,
    location: &'static str,
) -> HttpResponse {
    HttpResponse::Found()
        .cookie(
            Cookie::build(name, value)
                .path("/")
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(FIVE_YEARS_IN_SECS)
                .finish(),
        )
        .append_header((http::header::LOCATION, location.to_string()))
        .finish()
}

async fn login() -> impl Responder {
    set_cookie_and_redirect(
        KEEP_LOGGED_IN_COOKIE_NAME,
        KeepLoggedIn::Yes.into(),
        LOGIN_PATH,
    )
}

async fn logout() -> impl Responder {
    set_cookie_and_redirect(
        KEEP_LOGGED_IN_COOKIE_NAME,
        KeepLoggedIn::No.into(),
        LOGOUT_PATH,
    )
}

pub fn session_app() -> impl HttpServiceFactory {
    web::scope("/_")
        .route("/login", web::get().to(login))
        .route("/logout", web::get().to(logout))
}
