use crate::{db::DbPool, error::ApiError, models::NewMessage, queries};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// POST /api/messages
pub async fn create_message(
    State(pool): State<Arc<DbPool>>,
    Json(new_msg): Json<NewMessage>,
) -> Result<impl IntoResponse, ApiError> {
    queries::insert_message(&pool, &new_msg).await.map(Json)
}

/// GET /api/messages
pub async fn list_messages(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_messages(&pool).await.map(Json)
}

/// GET /api/servers/{id}/messages
pub async fn list_messages_by_server(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_messages_by_server(&pool, server_id).await.map(Json)
}

/// GET /api/channels/{id}/messages
pub async fn list_messages_by_channel(
    State(pool): State<Arc<DbPool>>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_messages_by_channel(&pool, channel_id).await.map(Json)
}

/// GET /api/messages/{id}
pub async fn get_message_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_message_by_id(&pool, message_id).await.map(Json)
}
