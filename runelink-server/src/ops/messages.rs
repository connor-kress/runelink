use crate::{error::ApiError, queries, state::AppState};
use runelink_types::NewMessage;
use uuid::Uuid;

/// Create a new message in a channel.
pub async fn create_message(
    state: &AppState,
    channel_id: Uuid,
    new_message: &NewMessage,
) -> Result<runelink_types::Message, ApiError> {
    queries::insert_message(&state.db_pool, channel_id, new_message).await
}

/// List all messages.
pub async fn list_messages(
    state: &AppState,
) -> Result<Vec<runelink_types::Message>, ApiError> {
    queries::get_all_messages(&state.db_pool).await
}

/// List messages in a server.
pub async fn list_messages_by_server(
    state: &AppState,
    server_id: Uuid,
) -> Result<Vec<runelink_types::Message>, ApiError> {
    queries::get_messages_by_server(&state.db_pool, server_id).await
}

/// List messages in a channel.
pub async fn list_messages_by_channel(
    state: &AppState,
    channel_id: Uuid,
) -> Result<Vec<runelink_types::Message>, ApiError> {
    queries::get_messages_by_channel(&state.db_pool, channel_id).await
}

/// Get a message by ID.
pub async fn get_message_by_id(
    state: &AppState,
    message_id: Uuid,
) -> Result<runelink_types::Message, ApiError> {
    queries::get_message_by_id(&state.db_pool, message_id).await
}
