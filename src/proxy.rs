use actix_web::client::Client;
use actix_web::error;
use actix_web::web::BytesMut;
use actix_web::Error;
use actix_web::HttpResponse;
use futures::Future;
use futures::Stream;
use serde_json::Value;

pub fn proxy(client: &Client, endpoint: &str) -> impl Future<Item = HttpResponse, Error = Error> {
    info!("proxying: {}", endpoint);
    client
        .get(endpoint)
        .send()
        .map_err(Error::from)
        .and_then(|res| {
            res.from_err()
                .fold(BytesMut::new(), |mut acc, chunk| {
                    acc.extend_from_slice(&chunk);
                    Ok::<_, Error>(acc)
                })
                .and_then(|body| {
                    serde_json::from_slice::<Value>(&body).map_err(error::ErrorBadRequest)
                })
                .map(|o| HttpResponse::Ok().json(o))
        })
        .map_err(error::ErrorBadRequest)
}
