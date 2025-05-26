use crate::error::CliError;
use reqwest::Client;
use runelink_types::Server;
use uuid::Uuid;

use super::fetch_json;

pub async fn fetch_servers(
    client: &Client,
    domain_api_base: &str,
) -> Result<Vec<Server>, CliError> {
    let url = format!("{}/servers", domain_api_base);
    fetch_json::<Vec<Server>>(client, &url).await
}

pub async fn fetch_server_by_id(
    client: &Client,
    domain_api_base: &str,
    server_id: Uuid,
) -> Result<Server, CliError> {
    let url = format!("{}/servers/{}", domain_api_base, server_id);
    fetch_json::<Server>(client, &url).await
}
