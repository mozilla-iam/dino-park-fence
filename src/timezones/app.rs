use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::Path;
use actix_web::Result;
use chrono::Offset;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Tz;
use serde_json::json;
use serde_json::Value;

const ALL_TIMEZONES_JSON_STR: &str = include_str!("../data/timezones.json");

fn list(_: &HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .header("content-type", "application/json")
        .body(ALL_TIMEZONES_JSON_STR))
}

fn offset(tz_str: Path<String>) -> Result<Json<Value>> {
    let tz: Tz = tz_str.parse().map_err(error::ErrorBadRequest)?;
    let tz_offset = tz.offset_from_utc_datetime(&Utc::now().naive_utc());
    let offset_to_utc = tz_offset.fix().local_minus_utc();
    Ok(Json(json!({ "offset_to_utc": offset_to_utc })))
}

pub fn timezone_app() -> App {
    App::new().prefix("/api/v4/timezone").configure(|app| {
        Cors::for_app(app)
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/list/", |r| r.f(list))
            .resource("/offset/{tz_str}", |r| {
                r.method(http::Method::GET).with(offset)
            })
            .register()
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_offset() {
        let path = Path::from(String::from("Europe/Berlin"));
        let res = offset(path);
        assert!(res.is_ok());
        let json = res.unwrap();
        let offset_to_utc = json.as_object().unwrap().get("offset_to_utc").unwrap();
        assert!(offset_to_utc == 3600 || offset_to_utc == 7200);
    }

    #[test]
    fn test_all_timezones_are_valid() {
        let all_timezones: Vec<&str> = serde_json::from_str(ALL_TIMEZONES_JSON_STR).unwrap();
        for timezone in all_timezones {
            assert!(timezone.parse::<Tz>().is_ok());
        }
    }
}
