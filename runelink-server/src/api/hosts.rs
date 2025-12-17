use crate::{error::ApiError, ops, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

/// GET /hosts
pub async fn list_hosts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let hosts = ops::list_hosts(&state).await?;
    Ok((StatusCode::OK, Json(hosts)))
}

/// GET /hosts/{domain}
pub async fn get_host(
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let host = ops::get_host(&state, &domain).await?;
    Ok((StatusCode::OK, Json(host)))
}
