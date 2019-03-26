use failure::Error;
use reqwest::Client;
use serde_json::json;

pub fn upload_picture(data_uri: &str, uuid: &str, fossil_send_endpoint: &str) -> Result<(), Error> {
    let payload = json!({ data_uri: data_uri });
    Client::new().post(fossil_send_endpoint).json(&payload).send()?.error_for_status()?;
    Ok(())
}