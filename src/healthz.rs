use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;

fn healthz(_: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn healthz_app() -> App {
    App::new().prefix("/healthz").configure(|app| {
        Cors::for_app(app)
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("", move |r| r.method(http::Method::GET).with(healthz))
            .register()
    })
}
