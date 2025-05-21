use crate::{db::DbPool, error::ApiError, models::NewChannel, queries};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

/// POST /api/servers/{channel_id}/channels
pub async fn create_channel(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
    Json(new_channel): Json<NewChannel>,
) -> Result<impl IntoResponse, ApiError> {
    queries::insert_channel(&pool, server_id, &new_channel).await.map(Json)
}

/// GET /api/channels
pub async fn list_channels(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_channels(&pool).await.map(Json)
}

/// GET /api/servers/{channel_id}/channels
pub async fn list_channels_by_server(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_channels_by_server(&pool, server_id).await.map(Json)
}

/// GET /api/channels/{channel_id}
pub async fn get_channel_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_channel_by_id(&pool, channel_id).await.map(Json)
}
