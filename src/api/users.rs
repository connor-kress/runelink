use crate::{db::DbPool, error::ApiError, models::NewUser, queries};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use uuid::Uuid;
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

/// GET /api/users/{id}
pub async fn get_user_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_user_by_id(&pool, user_id).await.map(Json)
}
