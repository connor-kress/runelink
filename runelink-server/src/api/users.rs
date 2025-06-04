use crate::{db::DbPool, error::ApiError, queries};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::NewUser;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct GetUserByNameDomainQuery {
    name: String,
    domain: String,
}

/// POST /api/users
pub async fn create_user(
    State(pool): State<Arc<DbPool>>,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    let user = queries::insert_user(&pool, &new_user).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// GET /api/users
pub async fn list_users(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_users(&pool).await.map(Json)
}

/// GET /api/users/{user_id}
pub async fn get_user_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_user_by_id(&pool, user_id).await.map(Json)
}

/// GET /api/users/find?name=...&domain=...
pub async fn find_user_by_name_domain_handler(
    State(pool): State<Arc<DbPool>>,
    Query(params): Query<GetUserByNameDomainQuery>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_user_by_name_and_domain(&pool, params.name, params.domain)
        .await
        .map(Json)
}

/// GET /api/users/{user_id}/domains
pub async fn get_user_associated_domains(
    State(pool): State<Arc<DbPool>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_associated_domains_for_user(&pool, user_id).await.map(Json)
}
