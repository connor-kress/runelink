use crate::{
    auth::{Principal, authorize},
    error::ApiError,
    ops,
    state::AppState,
};
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use runelink_types::NewMessage;
use uuid::Uuid;

/// POST /servers/{server_id}/channels/{channel_id}/messages
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
    Json(new_message): Json<NewMessage>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::messages::auth::create(server_id),
    )
    .await?;
    let message = ops::messages::create(
        &state,
        &session,
        server_id,
        channel_id,
        &new_message,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(message)))
}

/// GET /messages
pub async fn get_all(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::messages::auth::get_all(),
    )
    .await?;
    let messages = ops::messages::get_all(&state, &session).await?;
    Ok((StatusCode::OK, Json(messages)))
}

/// GET /servers/{server_id}/messages
pub async fn get_by_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::messages::auth::get_by_server(server_id),
    )
    .await?;
    let messages =
        ops::messages::get_by_server(&state, &session, server_id).await?;
    Ok((StatusCode::OK, Json(messages)))
}

/// GET /servers/{server_id}/channels/{channel_id}/messages
pub async fn get_by_channel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::messages::auth::get_by_channel(server_id),
    )
    .await?;
    let messages =
        ops::messages::get_by_channel(&state, &session, channel_id).await?;
    Ok((StatusCode::OK, Json(messages)))
}

/// GET /servers/{server_id}/channels/{channel_id}/messages/{message_id}
pub async fn get_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server_id, channel_id, message_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::messages::auth::get_by_id(server_id),
    )
    .await?;
    let message = ops::messages::get_by_id(
        &state, &session, server_id, channel_id, message_id,
    )
    .await?;
    Ok((StatusCode::OK, Json(message)))
}
