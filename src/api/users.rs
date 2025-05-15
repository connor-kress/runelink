use crate::{
    db::DbPool,
    db_queries::{get_all_users, insert_user},
    error::ApiError,
    models::NewUser,
};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

/// GET /api/users
pub async fn list_users(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    get_all_users(&pool).await.map(Json)
}

/// POST /api/users
pub async fn create_user(
    State(pool): State<Arc<DbPool>>,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    insert_user(&pool, &new_user).await.map(Json)
}
