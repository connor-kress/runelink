use runelink_types::{
    NewServer, NewServerMembership, Server, ServerRole, ServerWithChannels,
};
use uuid::Uuid;

use crate::{
    auth::{AuthSpec, Requirement, Session},
    error::ApiError,
    queries,
    state::AppState,
};

/// Create a new server (local resource) and add the creator as admin.
pub async fn create(
    state: &AppState,
    session: &Session,
    new_server: &NewServer,
) -> Result<Server, ApiError> {
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
    queries::memberships::insert(&state.db_pool, &new_membership).await?;
    Ok(server)
}

/// List all local servers (public).
pub async fn get_all(state: &AppState) -> Result<Vec<Server>, ApiError> {
    // TODO: add visibility specification for servers
    // We could then have an admin endpoint for all servers
    // and a public endpoint for only public servers
    let servers = queries::servers::get_all(state).await?;
    Ok(servers)
}

/// Get a server by ID (public).
pub async fn get_by_id(
    state: &AppState,
    server_id: Uuid,
) -> Result<Server, ApiError> {
    // TODO: separate public and private server objects?
    let server = queries::servers::get_by_id(state, server_id).await?;
    Ok(server)
}

/// Get a server with its channels.
pub async fn get_with_channels(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
) -> Result<ServerWithChannels, ApiError> {
    let (server, channels) = tokio::join!(
        queries::servers::get_by_id(state, server_id),
        queries::channels::get_by_server(&state.db_pool, server_id),
    );
    Ok(ServerWithChannels {
        server: server?,
        channels: channels?,
    })
}

/// Auth requirements for server operations.
pub mod auth {
    use super::*;

    pub fn create() -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::HostAdmin],
        }
    }

    pub fn get_with_channels(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
        }
    }
}
