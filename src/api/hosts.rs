use crate::{
    db::DbPool,
    queries::{get_all_hosts, get_host_by_domain},
    error::ApiError,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// GET /api/hosts
pub async fn list_hosts(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    get_all_hosts(&pool).await.map(Json)
}

/// GET /api/hosts/:domain
pub async fn get_host(
    State(pool): State<Arc<DbPool>>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    get_host_by_domain(&pool, &domain).await.map(Json)
}
