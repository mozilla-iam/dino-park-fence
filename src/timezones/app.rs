use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Result;

const ALL_TIMEZONES_JSON_STR: &str = include_str!("../data/timezones.json");

fn list(_: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .header("content-type", "application/json")
        .body(ALL_TIMEZONES_JSON_STR))
}

pub fn timezone_app() -> impl HttpServiceFactory {
    web::scope("/timezone")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("/list/").route(web::get().to(list)))
}

#[cfg(test)]
mod test {
    extern crate chrono_tz;

    use super::*;
    use chrono_tz::Tz;

    #[test]
    fn test_all_timezones_are_valid() {
        let all_timezones: Vec<&str> = serde_json::from_str(ALL_TIMEZONES_JSON_STR).unwrap();
        for timezone in all_timezones {
            assert!(timezone.parse::<Tz>().is_ok());
        }
    }
}
