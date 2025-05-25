use crate::error::CliError;
use reqwest::Client;
use runelink_types::Message;
use uuid::Uuid;

use super::fetch_json;

pub async fn fetch_all_messages(
    client: &Client,
    domain_api_base: &str,
) -> Result<Vec<Message>, CliError> {
    let url = format!("{}/messages", domain_api_base);
    fetch_json::<Vec<Message>>(client, &url).await
}

pub async fn fetch_message_by_id(
    client: &Client,
    domain_api_base: &str,
    message_id: Uuid,
) -> Result<Message, CliError> {
    let url = format!("{}/messages/{}", domain_api_base, message_id);
    fetch_json::<Message>(client, &url).await
}
