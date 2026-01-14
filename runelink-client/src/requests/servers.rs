use log::info;
use reqwest::Client;
use runelink_types::{NewServer, Server, ServerWithChannels};
use uuid::Uuid;

use crate::{error::Result, requests};

use super::{
    fetch_json, fetch_json_federated, post_json_authed, post_json_federated,
};

pub async fn create(
    client: &Client,
    api_url: &str,
    access_token: &str,
    new_server: &NewServer,
    target_domain: Option<&str>,
) -> Result<Server> {
    let mut url = format!("{api_url}/servers");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("creating server: {url}");
    post_json_authed::<_, Server>(client, &url, access_token, new_server).await
}

pub async fn fetch_all(
    client: &Client,
    api_url: &str,
    target_domain: Option<&str>,
) -> Result<Vec<Server>> {
    let mut url = format!("{api_url}/servers");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("fetching all servers: {url}");
    fetch_json::<Vec<Server>>(client, &url).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> Result<Server> {
    let mut url = format!("{api_url}/servers/{server_id}");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("fetching server: {url}");
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
    info!("converted memberships to servers");
    Ok(servers)
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// POST /federation/servers
    pub async fn create(
        client: &Client,
        api_url: &str,
        token: &str,
        new_server: &NewServer,
    ) -> Result<Server> {
        let url = format!("{api_url}/federation/servers");
        info!("creating server (federation): {url}");
        post_json_federated::<NewServer, Server>(
            client, &url, token, new_server,
        )
        .await
    }

    /// GET /federation/servers/{server_id}/with_channels
    pub async fn fetch_with_channels(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
    ) -> Result<ServerWithChannels> {
        let url =
            format!("{api_url}/federation/servers/{server_id}/with_channels");
        info!("fetching server with channels (federation): {url}");
        fetch_json_federated::<ServerWithChannels>(client, &url, token).await
    }
}
