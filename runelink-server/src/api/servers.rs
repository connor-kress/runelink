use crate::{auth::AuthBuilder, error::ApiError, ops, state::AppState};
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
    // TODO: get user id/session tokens
    // AuthBuilder::new(Some(new_server.user_id))
    let session = AuthBuilder::new().admin().build(&headers, &state).await?;
    let server = ops::create_server(&state, &session, &new_server).await?;
    Ok((StatusCode::CREATED, Json(server)))
}

/// GET /servers
pub async fn list_servers(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let servers = ops::list_servers(&state).await?;
    Ok((StatusCode::OK, Json(servers)))
}

/// GET /servers/{server_id}
pub async fn get_server_by_id_handler(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let server = ops::get_server_by_id(&state, server_id).await?;
    Ok((StatusCode::OK, Json(server)))
}

/// GET /servers/{server_id}/with_channels
pub async fn get_server_with_channels_handler(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let server = ops::get_server_with_channels(&state, server_id).await?;
    Ok((StatusCode::OK, Json(server)))
}

/// GET /users/{user_id}/servers
pub async fn list_server_memberships_by_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let memberships =
        ops::list_server_memberships_by_user(&state, user_id).await?;
    Ok((StatusCode::OK, Json(memberships)))
}
