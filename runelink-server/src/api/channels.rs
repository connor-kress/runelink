use crate::{error::ApiError, queries, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::NewChannel;
use uuid::Uuid;

/// POST /api/servers/{channel_id}/channels
pub async fn create_channel(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Json(new_channel): Json<NewChannel>,
) -> Result<impl IntoResponse, ApiError> {
    queries::insert_channel(&state.db_pool, server_id, &new_channel)
        .await
        .map(|channel| (StatusCode::CREATED, Json(channel)))
}

/// GET /api/channels
pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_channels(&state.db_pool).await.map(Json)
}

/// GET /api/servers/{channel_id}/channels
pub async fn list_channels_by_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_channels_by_server(&state.db_pool, server_id).await.map(Json)
}

/// GET /api/channels/{channel_id}
pub async fn get_channel_by_id_handler(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_channel_by_id(&state.db_pool, channel_id).await.map(Json)
}
