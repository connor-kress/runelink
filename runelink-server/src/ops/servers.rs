use crate::{error::ApiError, queries, state::AppState};
use runelink_types::{
    NewServer, NewServerMember, Server, ServerMembership, ServerWithChannels,
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
    let new_member =
        NewServerMember::admin(session.user.id, session.user.domain.clone());
    queries::add_user_to_server(&state.db_pool, server.id, &new_member).await?;
    Ok(server)
}

/// List all servers (local only for now).
pub async fn list_servers(state: &AppState) -> Result<Vec<Server>, ApiError> {
    queries::get_all_servers(state).await
}

/// Get a server by ID.
pub async fn get_server_by_id(
    state: &AppState,
    server_id: Uuid,
) -> Result<Server, ApiError> {
    queries::get_server_by_id(state, server_id).await
}

/// Get a server with its channels.
pub async fn get_server_with_channels(
    state: &AppState,
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

/// List all server memberships for a user.
pub async fn list_server_memberships_by_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>, ApiError> {
    queries::get_all_memberships_for_user(state, user_id).await
}
