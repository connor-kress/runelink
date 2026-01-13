use runelink_types::{Channel, NewChannel};
use uuid::Uuid;

use crate::{
    auth::{AuthSpec, Requirement, Session},
    error::ApiError,
    queries,
    state::AppState,
};

/// Create a new channel in a server.
pub async fn create(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
    new_channel: &NewChannel,
) -> Result<Channel, ApiError> {
    let channel =
        queries::channels::insert(&state.db_pool, server_id, new_channel)
            .await?;
    Ok(channel)
}

/// Get all channels.
pub async fn get_all(
    state: &AppState,
    _session: &Session,
) -> Result<Vec<Channel>, ApiError> {
    let channels = queries::channels::get_all(&state.db_pool).await?;
    Ok(channels)
}

/// Get channels in a server.
pub async fn get_by_server(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
) -> Result<Vec<Channel>, ApiError> {
    queries::channels::get_by_server(&state.db_pool, server_id).await
}

/// Get a channel by its ID.
pub async fn get_by_id(
    state: &AppState,
    _session: &Session,
    channel_id: Uuid,
) -> Result<Channel, ApiError> {
    let channel =
        queries::channels::get_by_id(&state.db_pool, channel_id).await?;
    Ok(channel)
}

/// Auth requirements for channel operations.
pub mod auth {
    use super::*;

    pub fn create(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerAdmin { server_id }],
        }
    }

    pub fn get_all() -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::HostAdmin],
        }
    }

    pub fn get_by_server(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
        }
    }

    pub fn get_by_id(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
        }
    }
}
