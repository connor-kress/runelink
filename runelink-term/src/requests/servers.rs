use reqwest::Client;
use runelink_types::Server;
use uuid::Uuid;

use crate::error::CliError;

use super::fetch_json;

pub async fn fetch_servers(
    client: &Client,
    api_base: &str,
) -> Result<Vec<Server>, CliError> {
    let url = format!("{}/servers", api_base);
    fetch_json::<Vec<Server>>(client, &url).await
}

pub async fn fetch_server_by_id(
    client: &Client,
    api_base: &str,
    server_id: Uuid,
) -> Result<Server, CliError> {
    let url = format!("{}/servers/{}", api_base, server_id);
    fetch_json::<Server>(client, &url).await
}
