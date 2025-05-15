use crate::{
    db::DbPool,
    db_queries::{get_all_messages, get_message_by_id},
    error::ApiError,
};
use axum::{extract::{Path, State}, response::IntoResponse, Json};
use uuid::Uuid;
use std::sync::Arc;

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
