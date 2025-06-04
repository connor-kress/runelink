use crate::{error::ApiError, queries, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::NewServerMember;
use uuid::Uuid;

/// POST /api/servers/{server_id}/users
pub async fn add_server_member(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Json(new_member): Json<NewServerMember>,
) -> Result<impl IntoResponse, ApiError> {
    queries::add_user_to_server(&state.db_pool, server_id, &new_member)
        .await
        .map(|member| (StatusCode::CREATED, Json(member)))
}

/// GET /api/servers/{server_id}/users
pub async fn list_server_members(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_server_members(&state.db_pool, server_id).await.map(Json)
}

/// GET /api/servers/{server_id}/users/{user_id}
pub async fn get_server_member(
    State(state): State<AppState>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_server_member(&state.db_pool, server_id, user_id)
        .await
        .map(Json)
}
