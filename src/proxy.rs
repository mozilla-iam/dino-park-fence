use actix_web::client::Client;
use actix_web::error;
use actix_web::Error;
use actix_web::HttpResponse;
use log::info;
use serde_json::Value;

pub async fn proxy(client: &Client, endpoint: &str) -> Result<HttpResponse, Error> {
    info!("proxying: {}", endpoint);
    let mut res = client
        .get(endpoint)
        .send()
        .await
        .map_err(error::ErrorBadRequest)?;
    let json = res.json::<Value>().await.map_err(error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(json))
}
