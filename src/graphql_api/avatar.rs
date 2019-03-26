use failure::Error;
use reqwest::Client;
use serde_json::json;

#[derive(Deserialize)]
struct UploadResponse {
    url: String,
}

pub fn upload_picture(
    data_uri: &str,
    uuid: &str,
    fossil_send_endpoint: &str,
) -> Result<String, Error> {
    let payload = json!({ data_uri: data_uri });
    let UploadResponse { url } = Client::new()
        .post(&format!("{}{}", fossil_send_endpoint, uuid))
        .json(&payload)
        .send()?
        .error_for_status()?
        .json()?;
    Ok(url)
}
