use crate::{
    db::DbPool,
    error::ApiError,
    models::NewMessage,
    queries::{get_all_messages, get_message_by_id, insert_message},
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// GET /api/messages
pub async fn list_messages(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    get_all_messages(&pool).await.map(Json)
}

/// GET /api/messages/:id
pub async fn get_message_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    get_message_by_id(&pool, message_id).await.map(Json)
}

/// POST /api/messages
pub async fn create_message(
    State(pool): State<Arc<DbPool>>,
    Json(new_msg): Json<NewMessage>,
) -> Result<impl IntoResponse, ApiError> {
    insert_message(&pool, &new_msg).await.map(Json)
}
