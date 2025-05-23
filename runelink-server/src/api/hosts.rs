use crate::{db::DbPool, error::ApiError, queries};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

/// GET /api/hosts
pub async fn list_hosts(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_hosts(&pool).await.map(Json)
}

/// GET /api/hosts/{domain}
pub async fn get_host(
    State(pool): State<Arc<DbPool>>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_host_by_domain(&pool, &domain).await.map(Json)
}
