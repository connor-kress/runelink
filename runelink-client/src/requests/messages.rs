use reqwest::Client;
use runelink_types::{Message, NewMessage};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json};

pub async fn create(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    channel_id: Uuid,
    new_message: &NewMessage,
) -> Result<Message> {
    let url =
        format!("{api_url}/servers/{server_id}/channels/{channel_id}/messages");
    post_json::<NewMessage, Message>(client, &url, new_message).await
}

#[allow(dead_code)]
pub async fn fetch_all(client: &Client, api_url: &str) -> Result<Vec<Message>> {
    let url = format!("{api_url}/messages");
    fetch_json::<Vec<Message>>(client, &url).await
}

#[allow(dead_code)]
pub async fn fetch_by_server(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
) -> Result<Vec<Message>> {
    let url = format!("{api_url}/servers/{server_id}/messages");
    fetch_json::<Vec<Message>>(client, &url).await
}

pub async fn fetch_by_channel(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    channel_id: Uuid,
) -> Result<Vec<Message>> {
    let url =
        format!("{api_url}/servers/{server_id}/channels/{channel_id}/messages");
    fetch_json::<Vec<Message>>(client, &url).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
) -> Result<Message> {
    let url = format!(
        "{api_url}/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
    );
    fetch_json::<Message>(client, &url).await
}
