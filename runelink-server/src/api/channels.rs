use crate::{error::ApiError, ops, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::NewChannel;
use uuid::Uuid;

/// POST /servers/{server_id}/channels
pub async fn create_channel(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Json(new_channel): Json<NewChannel>,
) -> Result<impl IntoResponse, ApiError> {
    let channel = ops::create_channel(&state, server_id, &new_channel).await?;
    Ok((StatusCode::CREATED, Json(channel)))
}

/// GET /channels
pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let channels = ops::list_channels(&state).await?;
    Ok((StatusCode::OK, Json(channels)))
}

/// GET /servers/{server_id}/channels
pub async fn list_channels_by_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let channels = ops::list_channels_by_server(&state, server_id).await?;
    Ok((StatusCode::OK, Json(channels)))
}

/// GET /channels/{channel_id}
pub async fn get_channel_by_id_handler(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let channel = ops::get_channel_by_id(&state, channel_id).await?;
    Ok((StatusCode::OK, Json(channel)))
}
