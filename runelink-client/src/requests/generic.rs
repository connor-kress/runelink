use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{Error, Result};

pub async fn fetch_text(
    client: &Client,
    url: &str,
) -> Result<String> {
    let response = client
        .get(url)
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response
            .text()
            .await
            .unwrap_or_else(
                |e| format!("Failed to get error message body: {}", e)
            );
        return Err(Error::Status(status, message));
    }
    let text_data = response.text().await?;
    Ok(text_data)
}

pub async fn fetch_json<T>(
    client: &Client,
    url: &str,
) -> Result<T>
where
    T: DeserializeOwned
{
    let response = client
        .get(url)
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response
            .text()
            .await
            .unwrap_or_else(
                |e| format!("Failed to get error message body: {}", e)
            );
        return Err(Error::Status(status, message));
    }
    let data = response.json::<T>().await?;
    Ok(data)
}

pub async fn post_json<I, O>(
    client: &Client,
    url: &str,
    request_body: &I,
) -> Result<O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let response = client
        .post(url)
        .json(request_body)
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response
            .text()
            .await
            .unwrap_or_else(
                |e| format!("Failed to get error message body: {}", e)
            );
        return Err(Error::Status(status, message));
    }
    let data = response.json::<O>().await?;
    Ok(data)
}
