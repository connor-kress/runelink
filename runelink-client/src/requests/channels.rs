use log::info;
use reqwest::Client;
use runelink_types::{Channel, NewChannel};
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
    new_channel: &NewChannel,
    target_host: Option<&str>,
) -> Result<Channel> {
    let mut url = format!("{api_url}/servers/{server_id}/channels");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("creating channel: {url}");
    post_json_authed::<NewChannel, Channel>(
        client,
        &url,
        access_token,
        new_channel,
    )
    .await
}

pub async fn fetch_all(
    client: &Client,
    api_url: &str,
    access_token: &str,
    target_host: Option<&str>,
) -> Result<Vec<Channel>> {
    let mut url = format!("{api_url}/channels");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("fetching all channels: {url}");
    fetch_json_authed::<Vec<Channel>>(client, &url, access_token).await
}

pub async fn fetch_by_server(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    target_host: Option<&str>,
) -> Result<Vec<Channel>> {
    let mut url = format!("{api_url}/servers/{server_id}/channels");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("fetching channels by server: {url}");
    fetch_json_authed::<Vec<Channel>>(client, &url, access_token).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    channel_id: Uuid,
    target_host: Option<&str>,
) -> Result<Channel> {
    let mut url =
        format!("{api_url}/servers/{server_id}/channels/{channel_id}");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("fetching channel: {url}");
    fetch_json_authed::<Channel>(client, &url, access_token).await
}

pub async fn delete(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    channel_id: Uuid,
    target_host: Option<&str>,
) -> Result<()> {
    let mut url =
        format!("{api_url}/servers/{server_id}/channels/{channel_id}");
    if let Some(host) = target_host {
        url = format!("{url}?target_host={host}");
    }
    info!("deleting channel: {url}");
    delete_authed(client, &url, access_token).await
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// POST /federation/servers/{server_id}/channels
    pub async fn create(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        new_channel: &NewChannel,
    ) -> Result<Channel> {
        let url = format!("{api_url}/federation/servers/{server_id}/channels");
        info!("creating channel (federation): {url}");
        post_json_federated::<NewChannel, Channel>(
            client,
            &url,
            token,
            new_channel,
        )
        .await
    }

    /// GET /federation/channels
    pub async fn fetch_all(
        client: &Client,
        api_url: &str,
        token: &str,
    ) -> Result<Vec<Channel>> {
        let url = format!("{api_url}/federation/channels");
        info!("fetching all channels (federation): {url}");
        fetch_json_federated::<Vec<Channel>>(client, &url, token).await
    }

    /// GET /federation/servers/{server_id}/channels
    pub async fn fetch_by_server(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
    ) -> Result<Vec<Channel>> {
        let url = format!("{api_url}/federation/servers/{server_id}/channels");
        info!("fetching channels by server (federation): {url}");
        fetch_json_federated::<Vec<Channel>>(client, &url, token).await
    }

    /// GET /federation/servers/{server_id}/channels/{channel_id}
    pub async fn fetch_by_id(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        channel_id: Uuid,
    ) -> Result<Channel> {
        let url = format!(
            "{api_url}/federation/servers/{server_id}/channels/{channel_id}"
        );
        info!("fetching channel (federation): {url}");
        fetch_json_federated::<Channel>(client, &url, token).await
    }

    /// DELETE /federation/servers/{server_id}/channels/{channel_id}
    pub async fn delete(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        channel_id: Uuid,
    ) -> Result<()> {
        let url = format!(
            "{api_url}/federation/servers/{server_id}/channels/{channel_id}"
        );
        info!("deleting channel (federation): {url}");
        delete_federated(client, &url, token).await
    }
}
