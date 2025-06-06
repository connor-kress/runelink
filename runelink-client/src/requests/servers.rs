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
    let url = format!("{}/servers", api_url);
    post_json::<NewServer, Server>(client, &url, new_server).await
}

pub async fn fetch_servers(
    client: &Client,
    api_url: &str,
) -> Result<Vec<Server>> {
    let url = format!("{}/servers", api_url);
    fetch_json::<Vec<Server>>(client, &url).await
}

pub async fn fetch_server_by_id(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
) -> Result<Server> {
    let url = format!("{}/servers/{}", api_url, server_id);
    fetch_json::<Server>(client, &url).await
}

pub async fn fetch_server_memberships_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>> {
    let url = format!("{}/users/{}/servers", api_url, user_id);
    fetch_json::<Vec<ServerMembership>>(client, &url).await
}

pub async fn fetch_servers_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<Server>> {
    let servers = fetch_server_memberships_by_user(
        client,
        &api_url,
        user_id,
    )
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
    let url = format!("{}/servers/{}/users", api_url, server_id);
    dbg!(&new_member);
    post_json::<NewServerMember, ServerMember>(client, &url, &new_member).await
}
