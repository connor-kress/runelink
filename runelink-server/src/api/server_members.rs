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
use runelink_types::{NewServerMember, ServerMembership};
use uuid::Uuid;

/// POST /servers/{server_id}/users
pub async fn add_server_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<Uuid>,
    Json(new_member): Json<NewServerMember>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::auth_add_server_member(server_id),
    )
    .await?;
    let member =
        ops::add_server_member(&state, &session, server_id, &new_member)
            .await?;
    Ok((StatusCode::CREATED, Json(member)))
}

/// GET /servers/{server_id}/users
pub async fn list_server_members(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let members = ops::list_server_members(&state, server_id).await?;
    Ok((StatusCode::OK, Json(members)))
}

/// GET /servers/{server_id}/users/{user_id}
pub async fn get_server_member(
    State(state): State<AppState>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let member = ops::get_server_member(&state, server_id, user_id).await?;
    Ok((StatusCode::OK, Json(member)))
}

/// POST /servers/{server_id}/remote-memberships
pub async fn create_remote_membership(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Json(membership): Json<ServerMembership>,
) -> Result<impl IntoResponse, ApiError> {
    let new_membership =
        ops::create_remote_membership(&state, server_id, &membership).await?;
    Ok((StatusCode::CREATED, Json(new_membership)))
}
