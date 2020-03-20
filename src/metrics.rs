use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use failure::Error;
use prometheus::Encoder;
use prometheus::IntCounter;
use prometheus::Registry;
use prometheus::TextEncoder;

#[derive(Clone)]
pub struct Counters {
    pub field_any_changed: IntCounter,
}

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub counters: Counters,
}

impl Metrics {
    pub fn new() -> Result<Self, Error> {
        let counters = Counters {
            field_any_changed: IntCounter::new("field_any_changed_counter", "field changed")?,
        };
        let registry = Registry::new();
        registry.register(Box::new(counters.field_any_changed.clone()))?;

        Ok(Metrics { registry, counters })
    }
}

async fn metrics(_: HttpRequest, m: web::Data<Metrics>) -> HttpResponse {
    let metric_families = m.registry.gather();
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => HttpResponse::Ok().body(buffer),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

pub fn metrics_app() -> impl HttpServiceFactory {
    web::scope("/metrics")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .finish(),
        )
        .service(web::resource("").to(metrics))
}
