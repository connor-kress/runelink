use reqwest::Client;
use runelink_types::{NewServer, Server};
use uuid::Uuid;

use crate::{error::Result, requests};

use super::{fetch_json, post_json};

pub async fn create(
    client: &Client,
    api_url: &str,
    new_server: &NewServer,
) -> Result<Server> {
    let url = format!("{api_url}/servers");
    post_json::<_, Server>(client, &url, new_server).await
}

pub async fn fetch_all(client: &Client, api_url: &str) -> Result<Vec<Server>> {
    let url = format!("{api_url}/servers");
    fetch_json::<Vec<Server>>(client, &url).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
) -> Result<Server> {
    let url = format!("{api_url}/servers/{server_id}");
    fetch_json::<Server>(client, &url).await
}

pub async fn fetch_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<Server>> {
    let servers =
        requests::memberships::fetch_by_user(client, api_url, user_id)
            .await?
            .into_iter()
            .map(|m| m.server)
            .collect();
    Ok(servers)
}
