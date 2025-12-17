use crate::{error::ApiError, queries, state::AppState};
use runelink_types::NewChannel;
use uuid::Uuid;

/// Create a new channel in a server.
pub async fn create_channel(
    state: &AppState,
    server_id: Uuid,
    new_channel: &NewChannel,
) -> Result<runelink_types::Channel, ApiError> {
    queries::insert_channel(&state.db_pool, server_id, new_channel).await
}

/// List all channels.
pub async fn list_channels(
    state: &AppState,
) -> Result<Vec<runelink_types::Channel>, ApiError> {
    queries::get_all_channels(&state.db_pool).await
}

/// List channels in a server.
pub async fn list_channels_by_server(
    state: &AppState,
    server_id: Uuid,
) -> Result<Vec<runelink_types::Channel>, ApiError> {
    queries::get_channels_by_server(&state.db_pool, server_id).await
}

/// Get a channel by ID.
pub async fn get_channel_by_id(
    state: &AppState,
    channel_id: Uuid,
) -> Result<runelink_types::Channel, ApiError> {
    queries::get_channel_by_id(&state.db_pool, channel_id).await
}
