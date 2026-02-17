use log::info;
use reqwest::Client;
use runelink_types::{Message, NewMessage};
use uuid::Uuid;

use crate::error::Result;

use super::{
    delete_authed, delete_federated, fetch_json_authed, fetch_json_federated,
    post_json_authed, post_json_federated,
};

pub async fn create(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    channel_id: Uuid,
    new_message: &NewMessage,
    target_host: Option<&str>,
) -> Result<Message> {
    let mut url =
        format!("{api_url}/servers/{server_id}/channels/{channel_id}/messages");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("creating message: {url}");
    post_json_authed::<NewMessage, Message>(
        client,
        &url,
        access_token,
        new_message,
    )
    .await
}

#[allow(dead_code)]
pub async fn fetch_all(
    client: &Client,
    api_url: &str,
    access_token: &str,
    target_host: Option<&str>,
) -> Result<Vec<Message>> {
    let mut url = format!("{api_url}/messages");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("fetching all messages: {url}");
    fetch_json_authed::<Vec<Message>>(client, &url, access_token).await
}

#[allow(dead_code)]
pub async fn fetch_by_server(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    target_host: Option<&str>,
) -> Result<Vec<Message>> {
    let mut url = format!("{api_url}/servers/{server_id}/messages");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("fetching messages by server: {url}");
    fetch_json_authed::<Vec<Message>>(client, &url, access_token).await
}

pub async fn fetch_by_channel(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    channel_id: Uuid,
    target_host: Option<&str>,
) -> Result<Vec<Message>> {
    let mut url =
        format!("{api_url}/servers/{server_id}/channels/{channel_id}/messages");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("fetching messages by channel: {url}");
    fetch_json_authed::<Vec<Message>>(client, &url, access_token).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
    target_host: Option<&str>,
) -> Result<Message> {
    let mut url = format!(
        "{api_url}/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
    );
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("fetching message: {url}");
    fetch_json_authed::<Message>(client, &url, access_token).await
}

pub async fn delete(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
    target_host: Option<&str>,
) -> Result<()> {
    let mut url = format!(
        "{api_url}/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
    );
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("deleting message: {url}");
    delete_authed(client, &url, access_token).await
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// POST /federation/servers/{server_id}/channels/{channel_id}/messages
    pub async fn create(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        channel_id: Uuid,
        new_message: &NewMessage,
    ) -> Result<Message> {
        let url = format!(
            "{api_url}/federation/servers/{server_id}/channels/{channel_id}/messages"
        );
        info!("creating message (federation): {url}");
        post_json_federated::<NewMessage, Message>(
            client,
            &url,
            token,
            new_message,
        )
        .await
    }

    /// GET /federation/messages
    pub async fn fetch_all(
        client: &Client,
        api_url: &str,
        token: &str,
    ) -> Result<Vec<Message>> {
        let url = format!("{api_url}/federation/messages");
        info!("fetching all messages (federation): {url}");
        fetch_json_federated::<Vec<Message>>(client, &url, token).await
    }

    /// GET /federation/servers/{server_id}/messages
    pub async fn fetch_by_server(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
    ) -> Result<Vec<Message>> {
        let url = format!("{api_url}/federation/servers/{server_id}/messages");
        info!("fetching messages by server (federation): {url}");
        fetch_json_federated::<Vec<Message>>(client, &url, token).await
    }

    /// GET /federation/servers/{server_id}/channels/{channel_id}/messages
    pub async fn fetch_by_channel(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        channel_id: Uuid,
    ) -> Result<Vec<Message>> {
        let url = format!(
            "{api_url}/federation/servers/{server_id}/channels/{channel_id}/messages"
        );
        info!("fetching messages by channel (federation): {url}");
        fetch_json_federated::<Vec<Message>>(client, &url, token).await
    }

    /// GET /federation/servers/{server_id}/channels/{channel_id}/messages/{message_id}
    pub async fn fetch_by_id(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    ) -> Result<Message> {
        let url = format!(
            "{api_url}/federation/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
        );
        info!("fetching message (federation): {url}");
        fetch_json_federated::<Message>(client, &url, token).await
    }

    /// DELETE /federation/servers/{server_id}/channels/{channel_id}/messages/{message_id}
    pub async fn delete(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        channel_id: Uuid,
        message_id: Uuid,
    ) -> Result<()> {
        let url = format!(
            "{api_url}/federation/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
        );
        info!("deleting message (federation): {url}");
        delete_federated(client, &url, token).await
    }
}
