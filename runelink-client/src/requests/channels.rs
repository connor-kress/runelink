use reqwest::Client;
use runelink_types::{Channel, NewChannel};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json};

pub async fn create_channel(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    new_channel: &NewChannel,
) -> Result<Channel> {
    let url = format!("{}/servers/{}/channels", api_url, server_id);
    post_json::<NewChannel, Channel>(client, &url, new_channel).await
}

pub async fn fetch_all_channels(
    client: &Client,
    api_url: &str,
) -> Result<Vec<Channel>> {
    let url = format!("{}/channels", api_url);
    fetch_json::<Vec<Channel>>(client, &url).await
}

pub async fn fetch_channels_by_server(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
) -> Result<Vec<Channel>> {
    let url = format!("{}/servers/{}/channels", api_url, server_id);
    fetch_json::<Vec<Channel>>(client, &url).await
}

pub async fn fetch_channel_by_id(
    client: &Client,
    api_url: &str,
    channel_id: Uuid,
) -> Result<Channel> {
    let url = format!("{}/channels/{}", api_url, channel_id);
    fetch_json::<Channel>(client, &url).await
}
