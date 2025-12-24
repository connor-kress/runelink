use reqwest::Client;
use runelink_types::{
    NewServer, NewServerMember, Server, ServerMember, ServerMembership,
};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json};

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

pub async fn join_server(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    new_member: &NewServerMember,
) -> Result<ServerMember> {
    let url = format!("{api_url}/servers/{server_id}/users");
    post_json::<_, ServerMember>(client, &url, new_member).await
}

pub async fn sync_remote_membership(
    client: &Client,
    api_url: &str,
    new_membership: &ServerMembership,
) -> Result<ServerMembership> {
    let server_id = new_membership.server.id;
    let url = format!("{api_url}/servers/{server_id}/remote-memberships");
    post_json::<_, ServerMembership>(client, &url, new_membership).await
}
