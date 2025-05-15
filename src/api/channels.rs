use crate::{
    db::DbPool,
    queries::get_channel_by_id,
    error::ApiError,
};
use axum::{extract::{Path, State}, response::IntoResponse, Json};
use uuid::Uuid;
use std::sync::Arc;

/// GET /api/channels/:id
pub async fn get_channel_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    get_channel_by_id(&pool, channel_id).await.map(Json)
}
