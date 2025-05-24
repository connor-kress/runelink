use crate::error::CliError;
use reqwest::Client;
use runelink_types::User;

pub async fn fetch_users(
    client: &Client,
    domain_api_base: &str,
) -> Result<Vec<User>, CliError> {
    let users_url = format!("{}/users", domain_api_base);
    let response = client
        .get(&users_url)
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
    let users = response.json::<Vec<User>>().await?;
    Ok(users)
}
