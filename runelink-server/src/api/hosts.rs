use crate::{error::ApiError, queries, state::AppState};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

/// GET /hosts
pub async fn list_hosts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_hosts(&state.db_pool).await.map(Json)
}

/// GET /hosts/{domain}
pub async fn get_host(
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_host_by_domain(&state.db_pool, &domain).await.map(Json)
}
