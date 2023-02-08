use crate::error::ApiError;
use cis_profile::schema::Display;
use failure::Error;
use reqwest::Client;

#[derive(Serialize)]
struct UploadRequest<'a> {
    data_uri: &'a str,
    display: &'a Display,
    old_url: Option<&'a str>,
}

#[derive(Serialize)]
struct SaveRequest<'a> {
    intermediate: &'a str,
    display: &'a Display,
    old_url: Option<&'a str>,
}

#[derive(Serialize)]
struct ChangeDisplayRequest<'a> {
    display: &'a Display,
    old_url: Option<&'a str>,
}

#[derive(Deserialize)]
struct UploadResponse {
    url: String,
}

pub async fn save_picture(
    update: &str,
    uuid: &str,
    display: &Display,
    old_url: Option<&str>,
    fossil_send_endpoint: &str,
) -> Result<String, Error> {
    if let Some(intermediate) = update.strip_prefix("intermediate:") {
        let payload = SaveRequest {
            intermediate,
            display,
            old_url,
        };
        let UploadResponse { url } = Client::new()
            .post(&format!("{fossil_send_endpoint}save/{uuid}"))
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(url)
    } else {
        Err(ApiError::Unknown.into())
    }
}

pub async fn change_picture_display(
    uuid: &str,
    display: &Display,
    old_url: Option<&str>,
    fossil_send_endpoint: &str,
) -> Result<String, Error> {
    let payload = ChangeDisplayRequest { display, old_url };
    let UploadResponse { url } = Client::new()
        .post(&format!("{fossil_send_endpoint}display/{uuid}"))
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(url)
}
