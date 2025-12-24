use super::Session;
use crate::{
    auth::{AuthSpec, Requirement},
    error::ApiError,
    queries,
    state::AppState,
};
use runelink_types::NewMessage;
use uuid::Uuid;

/// Auth requirements for `create_message`.
pub fn auth_create_message(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerMember { server_id }],
    }
}

/// Create a new message in a channel.
pub async fn create_message(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    new_message: &NewMessage,
) -> Result<runelink_types::Message, ApiError> {
    let channel =
        queries::get_channel_by_id(&state.db_pool, channel_id).await?;
    if channel.server_id != server_id {
        return Err(ApiError::AuthError(
            "Channel not found in specified server".into(),
        ));
    }
    let message =
        queries::insert_message(&state.db_pool, channel_id, new_message)
            .await?;
    Ok(message)
}

/// Auth requirements for `list_messages`.
pub fn auth_list_messages() -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::HostAdmin],
    }
}

/// List all messages.
pub async fn list_messages(
    state: &AppState,
    _session: &Session,
) -> Result<Vec<runelink_types::Message>, ApiError> {
    let messages = queries::get_all_messages(&state.db_pool).await?;
    Ok(messages)
}

/// Auth requirements for `list_messages_by_server`.
pub fn auth_list_messages_by_server(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerMember { server_id }],
    }
}

/// List messages in a server.
pub async fn list_messages_by_server(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
) -> Result<Vec<runelink_types::Message>, ApiError> {
    let messages =
        queries::get_messages_by_server(&state.db_pool, server_id).await?;
    Ok(messages)
}

/// Auth requirements for `list_messages_by_channel`.
pub fn auth_list_messages_by_channel(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerMember { server_id }],
    }
}

/// List messages in a channel.
pub async fn list_messages_by_channel(
    state: &AppState,
    _session: &Session,
    channel_id: Uuid,
) -> Result<Vec<runelink_types::Message>, ApiError> {
    let messages =
        queries::get_messages_by_channel(&state.db_pool, channel_id).await?;
    Ok(messages)
}

/// Auth requirements for `get_message_by_id`.
pub fn auth_get_message_by_id(server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::ServerMember { server_id }],
    }
}

/// Get a message by ID.
pub async fn get_message_by_id(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
) -> Result<runelink_types::Message, ApiError> {
    let message =
        queries::get_message_by_id(&state.db_pool, message_id).await?;
    if message.channel_id != channel_id {
        return Err(ApiError::AuthError(
            "Message not found in specified channel".into(),
        ));
    }
    let channel =
        queries::get_channel_by_id(&state.db_pool, channel_id).await?;
    if channel.server_id != server_id {
        return Err(ApiError::AuthError(
            "Message not found in specified server".into(),
        ));
    }
    Ok(message)
}
