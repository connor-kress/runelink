use crate::{error::ApiError, ops, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::NewMessage;
use uuid::Uuid;

/// POST /channels/{channel_id}/messages
pub async fn create_message(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    Json(new_message): Json<NewMessage>,
) -> Result<impl IntoResponse, ApiError> {
    let message = ops::create_message(&state, channel_id, &new_message).await?;
    Ok((StatusCode::CREATED, Json(message)))
}

/// GET /messages
pub async fn list_messages(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let messages = ops::list_messages(&state).await?;
    Ok((StatusCode::OK, Json(messages)))
}

/// GET /servers/{server_id}/messages
pub async fn list_messages_by_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let messages = ops::list_messages_by_server(&state, server_id).await?;
    Ok((StatusCode::OK, Json(messages)))
}

/// GET /channels/{channel_id}/messages
pub async fn list_messages_by_channel(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let messages = ops::list_messages_by_channel(&state, channel_id).await?;
    Ok((StatusCode::OK, Json(messages)))
}

/// GET /messages/{message_id}
pub async fn get_message_by_id_handler(
    State(state): State<AppState>,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let message = ops::get_message_by_id(&state, message_id).await?;
    Ok((StatusCode::OK, Json(message)))
}
