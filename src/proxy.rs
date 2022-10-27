use crate::error::ApiError;
use actix_web::HttpResponse;
use awc::Client;
use log::error;
use log::info;
use serde_json::Value;

const PAYLOAD_SIZE: usize = 2 * 1024 * 1024;

pub async fn proxy(client: &Client, endpoint: &str) -> Result<HttpResponse, ApiError> {
    info!("proxying: {}", endpoint);
    let mut res = client.get(endpoint).send().await.map_err(|e| {
        error!("proxy error: {}", e);
        ApiError::ProxyError
    })?;
    let json = res.json::<Value>().limit(PAYLOAD_SIZE).await.map_err(|e| {
        error!("proxy error: {}", e);
        ApiError::ProxyError
    })?;
    Ok(HttpResponse::Ok().json(json))
}
