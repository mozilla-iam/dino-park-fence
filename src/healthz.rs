use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;

fn healthz(_: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn healthz_app() -> impl HttpServiceFactory {
    web::scope("/healthz")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "HEAD"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("").to(healthz))
}
