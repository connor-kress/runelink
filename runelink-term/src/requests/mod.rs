pub mod generic;
pub mod messages;
pub mod users;
pub use generic::*;
pub use messages::*;
pub use users::*;

use crate::error::CliError;
use reqwest::Client;

pub async fn do_ping(
    client: &Client,
    domain_api_base: &str,
) -> Result<String, CliError> {
    let users_url = format!("{}/ping", domain_api_base);
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
    let message = response.text().await?;
    Ok(message)
}
