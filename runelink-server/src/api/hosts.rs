use crate::{error::ApiError, ops, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

/// GET /hosts
pub async fn get_all(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let hosts = ops::hosts::get_all(&state).await?;
    Ok((StatusCode::OK, Json(hosts)))
}

/// GET /hosts/{domain}
pub async fn get_by_domain(
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let host = ops::hosts::get_by_domain(&state, &domain).await?;
    Ok((StatusCode::OK, Json(host)))
}
