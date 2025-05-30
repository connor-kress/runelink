use crate::error::CliError;
use reqwest::Client;
use runelink_types::Channel;
use uuid::Uuid;

use super::fetch_json;

pub async fn fetch_all_channels(
    client: &Client,
    api_base: &str,
) -> Result<Vec<Channel>, CliError> {
    let url = format!("{}/channels", api_base);
    fetch_json::<Vec<Channel>>(client, &url).await
}

pub async fn fetch_channels_by_server(
    client: &Client,
    api_base: &str,
    server_id: Uuid,
) -> Result<Vec<Channel>, CliError> {
    let url = format!("{}/servers/{}/channels", api_base, server_id);
    fetch_json::<Vec<Channel>>(client, &url).await
}

pub async fn fetch_channel_by_id(
    client: &Client,
    api_base: &str,
    channel_id: Uuid,
) -> Result<Channel, CliError> {
    let url = format!("{}/channels/{}", api_base, channel_id);
    fetch_json::<Channel>(client, &url).await
}
