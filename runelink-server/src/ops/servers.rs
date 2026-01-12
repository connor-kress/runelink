use crate::{error::ApiError, queries, state::AppState};
use runelink_types::{
    NewServer, NewServerMembership, Server, ServerMembership, ServerRole,
    ServerWithChannels,
};
use uuid::Uuid;

use super::Session;
use crate::auth::{AuthSpec, Requirement};

/// Auth requirements for `create_server`.
pub fn auth_create_server() -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::HostAdmin],
    }
}

/// Create a new server (local resource) and add the creator as admin.
pub async fn create_server(
    state: &AppState,
    session: &Session,
    new_server: &NewServer,
) -> Result<Server, ApiError> {
    let server = queries::insert_server(state, new_server).await?;

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
    queries::add_user_to_server(&state.db_pool, &new_membership).await?;
    Ok(server)
}

/// List all local servers (public).
pub async fn list_servers(state: &AppState) -> Result<Vec<Server>, ApiError> {
    // TODO: add visibility specification for servers
    // We could then have an admin endpoint for all servers
    // and a public endpoint for only public servers
    let servers = queries::get_all_servers(state).await?;
    Ok(servers)
}

/// Get a server by ID (public).
pub async fn get_server_by_id(
    state: &AppState,
    server_id: Uuid,
) -> Result<Server, ApiError> {
    // TODO: separate public and private server objects?
    let server = queries::get_server_by_id(state, server_id).await?;
    Ok(server)
}

/// Auth requirements for `get_server_with_channels`.
pub fn auth_get_server_with_channels(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerMember { server_id }],
    }
}

/// Get a server with its channels.
pub async fn get_server_with_channels(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
) -> Result<ServerWithChannels, ApiError> {
    let (server, channels) = tokio::join!(
        queries::get_server_by_id(state, server_id),
        queries::get_channels_by_server(&state.db_pool, server_id),
    );
    Ok(ServerWithChannels {
        server: server?,
        channels: channels?,
    })
}

/// List all server memberships for a user (public).
pub async fn list_server_memberships_by_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>, ApiError> {
    let memberships =
        queries::get_all_memberships_for_user(state, user_id).await?;
    Ok(memberships)
}
