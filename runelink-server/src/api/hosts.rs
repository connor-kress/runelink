use crate::{error::ApiResult, ops, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::info;

/// GET /hosts
pub async fn get_all(
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    info!("GET /hosts");
    let hosts = ops::hosts::get_all(&state).await?;
    Ok((StatusCode::OK, Json(hosts)))
}

/// GET /hosts/{domain}
pub async fn get_by_domain(
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> ApiResult<impl IntoResponse> {
    info!("GET /hosts/{domain}");
    let host = ops::hosts::get_by_domain(&state, &domain).await?;
    Ok((StatusCode::OK, Json(host)))
}
