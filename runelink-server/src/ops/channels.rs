use super::Session;
use crate::{
    auth::{AuthSpec, Requirement},
    error::ApiError,
    queries,
    state::AppState,
};
use runelink_types::NewChannel;
use uuid::Uuid;

/// Auth requirements for `create_channel`.
pub fn auth_create_channel(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerAdmin { server_id }],
    }
}

/// Create a new channel in a server.
pub async fn create_channel(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
    new_channel: &NewChannel,
) -> Result<runelink_types::Channel, ApiError> {
    let channel =
        queries::insert_channel(&state.db_pool, server_id, new_channel).await?;
    Ok(channel)
}

/// Auth requirements for `list_channels`.
pub fn auth_list_channels() -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::HostAdmin],
    }
}

/// List all channels.
pub async fn list_channels(
    state: &AppState,
    _session: &Session,
) -> Result<Vec<runelink_types::Channel>, ApiError> {
    let channels = queries::get_all_channels(&state.db_pool).await?;
    Ok(channels)
}

/// Auth requirements for `list_channels_by_server`.
pub fn auth_list_channels_by_server(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerMember { server_id }],
    }
}

/// List channels in a server.
pub async fn list_channels_by_server(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
) -> Result<Vec<runelink_types::Channel>, ApiError> {
    queries::get_channels_by_server(&state.db_pool, server_id).await
}

/// Auth requirements for `get_channel_by_id`.
pub fn auth_get_channel_by_id(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerMember { server_id }],
    }
}

/// Get a channel by ID.
pub async fn get_channel_by_id(
    state: &AppState,
    _session: &Session,
    channel_id: Uuid,
) -> Result<runelink_types::Channel, ApiError> {
    let channel =
        queries::get_channel_by_id(&state.db_pool, channel_id).await?;
    Ok(channel)
}
