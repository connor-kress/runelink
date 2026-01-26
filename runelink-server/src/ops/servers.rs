use runelink_client::{requests, util::get_api_url};
use runelink_types::{
    NewServer, NewServerMembership, Server, ServerRole, ServerWithChannels,
};
use uuid::Uuid;

use crate::{
    auth::Session,
    error::{ApiError, ApiResult},
    queries,
    state::AppState,
};

/// Create a new server and add the creator as admin.
/// If target_domain is provided and not the local domain, creates on that remote domain.
/// Otherwise, creates locally.
pub async fn create(
    state: &AppState,
    session: &Session,
    new_server: &NewServer,
    target_domain: Option<&str>,
) -> ApiResult<Server> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let server = queries::servers::insert(state, new_server).await?;
        // Get the creator's user identity
        // Since this requires HostAdmin (which requires client auth), these fields are always present
        let user_ref = session.user_ref.clone().ok_or_else(|| {
            ApiError::Internal(
                "Session missing user identity for server creation".into(),
            )
        })?;
        let new_membership = NewServerMembership {
            user_id: user_ref.id,
            user_domain: user_ref.domain,
            server_id: server.id,
            server_domain: server.domain.clone(),
            role: ServerRole::Admin,
        };
        queries::memberships::insert_local(&state.db_pool, &new_membership)
            .await?;
        Ok(server)
    } else {
        // Create on remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated server creation"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let server = requests::servers::federated::create(
            &state.http_client,
            &api_url,
            &token,
            new_server,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to create server on {domain}: {e}"
            ))
        })?;
        Ok(server)
    }
}

/// List all servers (public).
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local servers.
pub async fn get_all(
    state: &AppState,
    target_domain: Option<&str>,
) -> ApiResult<Vec<Server>> {
    if !state.config.is_remote_domain(target_domain) {
        // Handle local case
        // TODO: add visibility specification for servers
        // We could then have an admin endpoint for all servers
        // and a public endpoint for only public servers
        let servers = queries::servers::get_all(state).await?;
        Ok(servers)
    } else {
        // Fetch from remote domain
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let servers =
            requests::servers::fetch_all(&state.http_client, &api_url, None)
                .await
                .map_err(|e| {
                    ApiError::Internal(format!(
                        "Failed to fetch servers from {domain}: {e}"
                    ))
                })?;
        Ok(servers)
    }
}

/// Get a server by ID (public).
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local server.
pub async fn get_by_id(
    state: &AppState,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<Server> {
    if !state.config.is_remote_domain(target_domain) {
        // Handle local case
        // TODO: separate public and private server objects?
        let server = queries::servers::get_by_id(state, server_id).await?;
        Ok(server)
    } else {
        // Fetch from remote domain
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let server = requests::servers::fetch_by_id(
            &state.http_client,
            &api_url,
            server_id,
            None,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch server from {domain}: {e}"
            ))
        })?;
        Ok(server)
    }
}

/// Get a server with its channels.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local server with channels.
pub async fn get_with_channels(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<ServerWithChannels> {
    if !state.config.is_remote_domain(target_domain) {
        // Handle local case
        let (server, channels) = tokio::join!(
            queries::servers::get_by_id(state, server_id),
            queries::channels::get_by_server(&state.db_pool, server_id),
        );
        Ok(ServerWithChannels {
            server: server?,
            channels: channels?,
        })
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated server fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let server_with_channels =
            requests::servers::federated::fetch_with_channels(
                &state.http_client,
                &api_url,
                &token,
                server_id,
            )
            .await
            .map_err(|e| {
                ApiError::Internal(format!(
                    "Failed to fetch server with channels from {domain}: {e}"
                ))
            })?;
        Ok(server_with_channels)
    }
}

/// Delete a server by ID.
/// If target_domain is provided and not the local domain, deletes on that remote domain.
/// Otherwise, deletes locally.
pub async fn delete(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<()> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        queries::servers::delete(state, server_id).await?;
        Ok(())
    } else {
        // Delete on remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated server deletion"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        requests::servers::federated::delete(
            &state.http_client,
            &api_url,
            &token,
            server_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to delete server on {domain}: {e}"
            ))
        })?;
        Ok(())
    }
}

/// Auth requirements for server operations.
pub mod auth {
    use super::*;
    use crate::and;
    use crate::auth::Requirement as Req;

    pub fn create() -> Req {
        // TODO: add rate limiting or something
        and!(Req::Client)
    }

    pub fn get_with_channels(server_id: Uuid) -> Req {
        and!(Req::Client, Req::ServerMember { server_id })
    }

    pub fn delete(server_id: Uuid) -> Req {
        and!(Req::Client, Req::ServerAdmin { server_id })
    }

    pub mod federated {
        use super::*;

        pub fn create() -> Req {
            and!(Req::Federation, Req::HostAdmin)
        }

        pub fn get_with_channels(server_id: Uuid) -> Req {
            and!(Req::Federation, Req::ServerMember { server_id })
        }

        pub fn delete(server_id: Uuid) -> Req {
            and!(Req::Federation, Req::ServerAdmin { server_id })
        }
    }
}
