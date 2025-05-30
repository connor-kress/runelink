use crate::error::CliError;
use reqwest::Client;
use runelink_types::Message;
use uuid::Uuid;

use super::fetch_json;

pub async fn fetch_all_messages(
    client: &Client,
    api_base: &str,
) -> Result<Vec<Message>, CliError> {
    let url = format!("{}/messages", api_base);
    fetch_json::<Vec<Message>>(client, &url).await
}

pub async fn fetch_messages_by_server(
    client: &Client,
    api_base: &str,
    server_id: Uuid,
) -> Result<Vec<Message>, CliError> {
    let url = format!("{}/servers/{}/messages", api_base, server_id);
    fetch_json::<Vec<Message>>(client, &url).await
}

pub async fn fetch_messages_by_channel(
    client: &Client,
    api_base: &str,
    channel_id: Uuid,
) -> Result<Vec<Message>, CliError> {
    let url = format!("{}/channels/{}/messages", api_base, channel_id);
    fetch_json::<Vec<Message>>(client, &url).await
}

pub async fn fetch_message_by_id(
    client: &Client,
    api_base: &str,
    message_id: Uuid,
) -> Result<Message, CliError> {
    let url = format!("{}/messages/{}", api_base, message_id);
    fetch_json::<Message>(client, &url).await
}
