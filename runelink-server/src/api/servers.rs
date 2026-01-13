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
use runelink_types::NewServer;
use uuid::Uuid;

/// POST /servers
pub async fn create_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(new_server): Json<NewServer>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::servers::auth::create(),
    )
    .await?;
    let server = ops::servers::create(&state, &session, &new_server).await?;
    Ok((StatusCode::CREATED, Json(server)))
}

/// GET /servers
pub async fn list_servers(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let servers = ops::servers::get_all(&state).await?;
    Ok((StatusCode::OK, Json(servers)))
}

/// GET /servers/{server_id}
pub async fn get_server_by_id_handler(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let server = ops::servers::get_by_id(&state, server_id).await?;
    Ok((StatusCode::OK, Json(server)))
}

/// GET /servers/{server_id}/with_channels
pub async fn get_server_with_channels_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::servers::auth::get_with_channels(server_id),
    )
    .await?;
    let server_with_channels =
        ops::servers::get_with_channels(&state, &session, server_id).await?;
    Ok((StatusCode::OK, Json(server_with_channels)))
}

/// GET /users/{user_id}/servers
pub async fn list_server_memberships_by_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let memberships = ops::memberships::get_by_user(&state, user_id).await?;
    Ok((StatusCode::OK, Json(memberships)))
}
