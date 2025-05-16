use crate::{db::DbPool, error::ApiError, models::NewUser, queries};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

/// POST /api/users
pub async fn create_user(
    State(pool): State<Arc<DbPool>>,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    queries::insert_user(&pool, &new_user).await.map(Json)
}

/// GET /api/users
pub async fn list_users(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_users(&pool).await.map(Json)
}
