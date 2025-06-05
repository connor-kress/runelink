use reqwest::Client;
use runelink_types::{
    NewServer, NewServerMember, Server, ServerMember, ServerMembership,
};
use uuid::Uuid;

use crate::error::CliError;

use super::{fetch_json, post_json};

pub async fn create_server(
    client: &Client,
    api_base: &str,
    new_server: &NewServer,
) -> Result<Server, CliError> {
    let url = format!("{}/servers", api_base);
    post_json::<NewServer, Server>(client, &url, new_server).await
}

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

pub async fn fetch_server_memberships_by_user(
    client: &Client,
    api_base: &str,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>, CliError> {
    let url = format!("{}/users/{}/servers", api_base, user_id);
    fetch_json::<Vec<ServerMembership>>(client, &url).await
}

pub async fn fetch_servers_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<Server>, CliError> {
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
    api_base: &str,
    server_id: Uuid,
    new_member: &NewServerMember,
) -> Result<ServerMember, CliError> {
    let url = format!("{}/servers/{}/users", api_base, server_id);
    post_json::<NewServerMember, ServerMember>(client, &url, &new_member).await
}
