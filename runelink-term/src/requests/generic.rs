use crate::error::CliError;
use reqwest::Client;
use serde::de::DeserializeOwned;

pub async fn fetch_json<T>(
    client: &Client,
    url: &str,
) -> Result<T, CliError>
where
    T: DeserializeOwned
{
    let response = client
        .get(url)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to get error message body".to_string());
        return Err(CliError::ApiStatusError { status, message });
    }
    let data = response.json::<T>().await?;
    Ok(data)
}
