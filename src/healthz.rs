use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpResponse;

async fn healthz() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn healthz_app() -> impl HttpServiceFactory {
    web::scope("/healthz").service(web::resource("").to(healthz))
}
