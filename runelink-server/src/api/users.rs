use crate::{
    auth::{Principal, authorize},
    error::ApiError,
    ops,
    state::AppState,
};
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use runelink_types::NewUser;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct GetUserByNameDomainQuery {
    name: String,
    domain: String,
}

/// POST /users
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::users::auth::create(),
    )
    .await?;
    let user = ops::users::create(&state, &session, &new_user).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// GET /users
pub async fn get_all(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let users = ops::users::get_all(&state).await?;
    Ok((StatusCode::OK, Json(users)))
}

/// GET /users/{user_id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let user = ops::users::get_by_id(&state, user_id).await?;
    Ok((StatusCode::OK, Json(user)))
}

/// GET /users/find?name=...&domain=...
pub async fn get_by_name_and_domain(
    State(state): State<AppState>,
    Query(params): Query<GetUserByNameDomainQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let user =
        ops::users::get_by_name_and_domain(&state, params.name, params.domain)
            .await?;
    Ok((StatusCode::OK, Json(user)))
}

/// GET /users/{user_id}/domains
pub async fn get_user_associated_domains(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let domains =
        ops::hosts::get_user_associated_domains(&state, user_id).await?;
    Ok((StatusCode::OK, Json(domains)))
}
