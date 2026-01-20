use log::info;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

use crate::error::{Error, Result};

pub async fn fetch_text(client: &Client, url: &str) -> Result<String> {
    info!("fetching text: {url}");
    let response = client.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    let text_data = response.text().await?;
    Ok(text_data)
}

pub async fn fetch_json<T>(client: &Client, url: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    info!("fetching json: {url}");
    let response = client.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
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
    info!(
        "posting json: {url}\n{}",
        serde_json::to_string_pretty(request_body).unwrap()
    );
    let response = client.post(url).json(request_body).send().await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    let data = response.json::<O>().await?;
    Ok(data)
}

/// Helper to fetch JSON with federation auth token.
pub async fn fetch_json_federated<T>(
    client: &Client,
    url: &str,
    token: &str,
) -> Result<T>
where
    T: DeserializeOwned,
{
    info!("fetching json (federation): {url}");
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    let data = response.json::<T>().await?;
    Ok(data)
}

/// Helper to post JSON with federation auth token.
pub async fn post_json_federated<I, O>(
    client: &Client,
    url: &str,
    token: &str,
    request_body: &I,
) -> Result<O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    info!(
        "posting json (federation): {url}\n{}",
        serde_json::to_string_pretty(request_body).unwrap()
    );
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {token}"))
        .json(request_body)
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    let data = response.json::<O>().await?;
    Ok(data)
}

/// Helper to fetch text with client access token.
pub async fn fetch_text_authed(
    client: &Client,
    url: &str,
    access_token: &str,
) -> Result<String> {
    info!("fetching text (authenticated): {url}");
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    let text_data = response.text().await?;
    Ok(text_data)
}

/// Helper to fetch JSON with client access token.
pub async fn fetch_json_authed<T>(
    client: &Client,
    url: &str,
    access_token: &str,
) -> Result<T>
where
    T: DeserializeOwned,
{
    info!("fetching json (authenticated): {url}");
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    let data = response.json::<T>().await?;
    Ok(data)
}

/// Helper to post JSON with client access token.
pub async fn post_json_authed<I, O>(
    client: &Client,
    url: &str,
    access_token: &str,
    request_body: &I,
) -> Result<O>
where
    I: Serialize,
    O: DeserializeOwned,
{
    info!(
        "posting json (authenticated): {url}\n{}",
        serde_json::to_string_pretty(request_body).unwrap()
    );
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {access_token}"))
        .json(request_body)
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    let data = response.json::<O>().await?;
    Ok(data)
}

/// Helper to delete with client access token.
pub async fn delete_authed(
    client: &Client,
    url: &str,
    access_token: &str,
) -> Result<()> {
    info!("deleting (authenticated): {url}");
    let response = client
        .delete(url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    Ok(())
}

/// Helper to delete with federation auth token.
pub async fn delete_federated(
    client: &Client,
    url: &str,
    token: &str,
) -> Result<()> {
    info!("deleting (federation): {url}");
    let response = client
        .delete(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_else(|e| {
            format!("Failed to get error message body: {e}")
        });
        return Err(Error::Status(status, message));
    }
    Ok(())
}
