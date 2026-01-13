use log::info;
use reqwest::Client;
use runelink_types::{Channel, NewChannel};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json_authed, post_json_authed};

pub async fn create(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    new_channel: &NewChannel,
) -> Result<Channel> {
    let url = format!("{api_url}/servers/{server_id}/channels");
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
) -> Result<Vec<Channel>> {
    let url = format!("{api_url}/channels");
    info!("fetching all channels: {url}");
    fetch_json_authed::<Vec<Channel>>(client, &url, access_token).await
}

pub async fn fetch_by_server(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
) -> Result<Vec<Channel>> {
    let url = format!("{api_url}/servers/{server_id}/channels");
    info!("fetching channels by server: {url}");
    fetch_json_authed::<Vec<Channel>>(client, &url, access_token).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    access_token: &str,
    server_id: Uuid,
    channel_id: Uuid,
) -> Result<Channel> {
    let url = format!("{api_url}/servers/{server_id}/channels/{channel_id}");
    info!("fetching channel: {url}");
    fetch_json_authed::<Channel>(client, &url, access_token).await
}
