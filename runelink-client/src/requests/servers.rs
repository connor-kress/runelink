use reqwest::Client;
use runelink_types::{
    FullServerMembership, NewServer, NewServerMembership, Server,
    ServerMembership,
};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json, post_json_federated};

pub async fn create_server(
    client: &Client,
    api_url: &str,
    new_server: &NewServer,
) -> Result<Server> {
    let url = format!("{api_url}/servers");
    post_json::<_, Server>(client, &url, new_server).await
}

pub async fn fetch_servers(
    client: &Client,
    api_url: &str,
) -> Result<Vec<Server>> {
    let url = format!("{api_url}/servers");
    fetch_json::<Vec<Server>>(client, &url).await
}

pub async fn fetch_server_by_id(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
) -> Result<Server> {
    let url = format!("{api_url}/servers/{server_id}");
    fetch_json::<Server>(client, &url).await
}

pub async fn fetch_server_memberships_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>> {
    let url = format!("{api_url}/users/{user_id}/servers");
    fetch_json::<Vec<ServerMembership>>(client, &url).await
}

pub async fn fetch_servers_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<Server>> {
    let servers = fetch_server_memberships_by_user(client, api_url, user_id)
        .await?
        .into_iter()
        .map(|m| m.server)
        .collect();
    Ok(servers)
}

pub async fn create_membership(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    new_membership: &NewServerMembership,
) -> Result<FullServerMembership> {
    let url = format!("{api_url}/servers/{server_id}/users");
    post_json::<NewServerMembership, FullServerMembership>(
        client,
        &url,
        new_membership,
    )
    .await
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// Create a remote membership via federation (requires federation JWT).
    ///
    /// POST /federation/servers/{server_id}/users
    pub async fn create_membership(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        new_membership: &NewServerMembership,
    ) -> Result<FullServerMembership> {
        let url = format!("{api_url}/federation/servers/{server_id}/users");
        post_json_federated::<NewServerMembership, FullServerMembership>(
            client,
            &url,
            token,
            new_membership,
        )
        .await
    }
}
