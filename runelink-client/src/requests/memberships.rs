use log::info;
use reqwest::Client;
use runelink_types::{
    FullServerMembership, NewServerMembership, ServerMembership,
};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json, post_json_federated};

pub async fn fetch_by_user(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>> {
    let url = format!("{api_url}/users/{user_id}/servers");
    info!("fetching memberships by user: {url}");
    fetch_json::<Vec<ServerMembership>>(client, &url).await
}

pub async fn create(
    client: &Client,
    api_url: &str,
    server_id: Uuid,
    new_membership: &NewServerMembership,
) -> Result<FullServerMembership> {
    let url = format!("{api_url}/servers/{server_id}/users");
    info!("creating membership: {url}");
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
    pub async fn create(
        client: &Client,
        api_url: &str,
        token: &str,
        server_id: Uuid,
        new_membership: &NewServerMembership,
    ) -> Result<FullServerMembership> {
        let url = format!("{api_url}/federation/servers/{server_id}/users");
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
