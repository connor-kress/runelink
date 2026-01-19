use log::info;
use reqwest::Client;
use runelink_types::{
    FullServerMembership, NewServerMembership, ServerMembership,
};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json_authed, post_json_federated};

pub async fn fetch_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>> {
    let url = format!("{api_url}/users/{user_id}/servers");
    info!("fetching memberships by user: {url}");
    fetch_json::<Vec<ServerMembership>>(client, &url).await
}

pub async fn fetch_members_by_server(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> Result<Vec<runelink_types::ServerMember>> {
    let mut url = format!("{api_url}/servers/{server_id}/users");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("fetching members by server: {url}");
    fetch_json::<Vec<runelink_types::ServerMember>>(client, &url).await
}

pub async fn fetch_member_by_user_and_server(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    user_id: Uuid,
    target_domain: Option<&str>,
) -> Result<runelink_types::ServerMember> {
    let mut url = format!("{api_url}/servers/{server_id}/users/{user_id}");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("fetching member by user and server: {url}");
    fetch_json::<runelink_types::ServerMember>(client, &url).await
}

pub async fn create(
    client: &Client,
    api_url: &str,
    access_token: &str,
    new_membership: &NewServerMembership,
) -> Result<FullServerMembership> {
    let url = format!(
        "{api_url}/servers/{server_id}/users",
        server_id = new_membership.server_id
    );
    info!("creating membership: {url}");
    post_json_authed::<NewServerMembership, FullServerMembership>(
        client,
        &url,
        access_token,
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
    pub async fn create(
        client: &Client,
        api_url: &str,
        token: &str,
        new_membership: &NewServerMembership,
    ) -> Result<FullServerMembership> {
        let url = format!(
            "{api_url}/federation/servers/{server_id}/users",
            server_id = new_membership.server_id
        );
        info!("creating membership (federation): {url}");
        post_json_federated::<NewServerMembership, FullServerMembership>(
            client,
            &url,
            token,
            new_membership,
        )
        .await
    }
}
