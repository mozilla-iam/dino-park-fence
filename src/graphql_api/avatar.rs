use cis_profile::schema::Display;
use failure::Error;
use reqwest::blocking::Client;

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

pub fn upload_picture(
    update: &str,
    uuid: &str,
    display: &Display,
    old_url: Option<&str>,
    fossil_send_endpoint: &str,
) -> Result<String, Error> {
    if update.starts_with("intermediate:") {
        // TODO: clean up once migrated.
        let payload = SaveRequest {
            intermediate: &update[13..],
            display,
            old_url,
        };
        let UploadResponse { url } = Client::new()
            .post(&format!("{}save/{}", fossil_send_endpoint, uuid))
            .json(&payload)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(url)
    } else {
        let payload = UploadRequest {
            data_uri: update,
            display,
            old_url,
        };
        let UploadResponse { url } = Client::new()
            .post(&format!("{}{}", fossil_send_endpoint, uuid))
            .json(&payload)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(url)
    }
}

pub fn change_picture_display(
    uuid: &str,
    display: &Display,
    old_url: Option<&str>,
    fossil_send_endpoint: &str,
) -> Result<String, Error> {
    let payload = ChangeDisplayRequest { display, old_url };
    let UploadResponse { url } = Client::new()
        .post(&format!("{}display/{}", fossil_send_endpoint, uuid))
        .json(&payload)
        .send()?
        .error_for_status()?
        .json()?;
    Ok(url)
}
