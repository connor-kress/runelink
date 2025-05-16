use crate::{db::DbPool, error::ApiError, queries};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// GET /api/channels/{id}
pub async fn get_channel_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_channel_by_id(&pool, channel_id).await.map(Json)
}
