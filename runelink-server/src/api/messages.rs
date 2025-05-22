use crate::{db::DbPool, error::ApiError, queries};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::NewMessage;
use std::sync::Arc;
use uuid::Uuid;

/// POST /api/channels/{channel_id}/messages
pub async fn create_message(
    State(pool): State<Arc<DbPool>>,
    Path(channel_id): Path<Uuid>,
    Json(new_message): Json<NewMessage>,
) -> Result<impl IntoResponse, ApiError> {
    queries::insert_message(&pool, channel_id, &new_message)
        .await
        .map(|message| (StatusCode::CREATED, Json(message)))
}

/// GET /api/messages
pub async fn list_messages(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_messages(&pool).await.map(Json)
}

/// GET /api/servers/{server_id}/messages
pub async fn list_messages_by_server(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_messages_by_server(&pool, server_id).await.map(Json)
}

/// GET /api/channels/{channel_id}/messages
pub async fn list_messages_by_channel(
    State(pool): State<Arc<DbPool>>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_messages_by_channel(&pool, channel_id).await.map(Json)
}

/// GET /api/messages/{message_id}
pub async fn get_message_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_message_by_id(&pool, message_id).await.map(Json)
}
