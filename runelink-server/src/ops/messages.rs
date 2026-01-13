use runelink_types::{Message, NewMessage};
use uuid::Uuid;

use crate::{
    auth::{AuthSpec, Requirement, Session},
    error::ApiError,
    queries,
    state::AppState,
};

/// Create a new message in a channel.
pub async fn create(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    new_message: &NewMessage,
) -> Result<Message, ApiError> {
    let channel =
        queries::channels::get_by_id(&state.db_pool, channel_id).await?;
    if channel.server_id != server_id {
        return Err(ApiError::AuthError(
            "Channel not found in specified server".into(),
        ));
    }
    let message =
        queries::messages::insert(&state.db_pool, channel_id, new_message)
            .await?;
    Ok(message)
}

/// Get all messages.
pub async fn get_all(
    state: &AppState,
    _session: &Session,
) -> Result<Vec<Message>, ApiError> {
    let messages = queries::messages::get_all(&state.db_pool).await?;
    Ok(messages)
}

/// Get messages in a server.
pub async fn get_by_server(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
) -> Result<Vec<Message>, ApiError> {
    let messages =
        queries::messages::get_by_server(&state.db_pool, server_id).await?;
    Ok(messages)
}

/// Get messages in a channel.
pub async fn get_by_channel(
    state: &AppState,
    _session: &Session,
    channel_id: Uuid,
) -> Result<Vec<Message>, ApiError> {
    let messages =
        queries::messages::get_by_channel(&state.db_pool, channel_id).await?;
    Ok(messages)
}

/// Get a message by its ID.
pub async fn get_by_id(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
) -> Result<Message, ApiError> {
    let message =
        queries::messages::get_by_id(&state.db_pool, message_id).await?;
    if message.channel_id != channel_id {
        return Err(ApiError::AuthError(
            "Message not found in specified channel".into(),
        ));
    }
    let channel =
        queries::channels::get_by_id(&state.db_pool, channel_id).await?;
    if channel.server_id != server_id {
        return Err(ApiError::AuthError(
            "Message not found in specified server".into(),
        ));
    }
    Ok(message)
}

/// Auth requirements for message operations.
pub mod auth {
    use super::*;

    pub fn create(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
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

    pub fn get_by_channel(server_id: Uuid) -> AuthSpec {
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
