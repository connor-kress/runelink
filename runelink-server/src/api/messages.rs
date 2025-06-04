use crate::{error::ApiError, queries, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::NewMessage;
use uuid::Uuid;

/// POST /api/channels/{channel_id}/messages
pub async fn create_message(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    Json(new_message): Json<NewMessage>,
) -> Result<impl IntoResponse, ApiError> {
    queries::insert_message(&state.db_pool, channel_id, &new_message)
        .await
        .map(|message| (StatusCode::CREATED, Json(message)))
}

/// GET /api/messages
pub async fn list_messages(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_messages(&state.db_pool).await.map(Json)
}

/// GET /api/servers/{server_id}/messages
pub async fn list_messages_by_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_messages_by_server(&state.db_pool, server_id).await.map(Json)
}

/// GET /api/channels/{channel_id}/messages
pub async fn list_messages_by_channel(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_messages_by_channel(&state.db_pool, channel_id).await.map(Json)
}

/// GET /api/messages/{message_id}
pub async fn get_message_by_id_handler(
    State(state): State<AppState>,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_message_by_id(&state.db_pool, message_id).await.map(Json)
}
