use crate::{db::DbPool, error::ApiError, models::NewServerMember, queries};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

/// POST /api/servers/{server_id}/users
pub async fn add_server_member(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
    Json(new_member): Json<NewServerMember>,
) -> Result<impl IntoResponse, ApiError> {
    queries::add_user_to_server(&pool, server_id, &new_member)
        .await
        .map(|member| (StatusCode::CREATED, Json(member)))
}

/// GET /api/servers/{server_id}/users
pub async fn list_server_members(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_server_members(&pool, server_id).await.map(Json)
}

/// GET /api/servers/{server_id}/users/{user_id}
pub async fn get_server_member(
    State(pool): State<Arc<DbPool>>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_server_member(&pool, server_id, user_id).await.map(Json)
}
