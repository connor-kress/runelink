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
use log::info;
use runelink_types::NewUser;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct GetUserByNameDomainQuery {
    name: String,
    domain: String,
}

#[derive(Deserialize, Debug)]
pub struct UserQueryParams {
    pub target_domain: Option<String>,
}

/// POST /users
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    info!("POST /users\nnew_user = {:#?}", new_user);
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
    Query(params): Query<UserQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    info!("GET /users?target_domain={:?}", params.target_domain);
    let users =
        ops::users::get_all(&state, params.target_domain.as_deref()).await?;
    Ok((StatusCode::OK, Json(users)))
}

/// GET /users/{user_id}
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<UserQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    info!(
        "GET /users/{user_id}?target_domain={:?}",
        params.target_domain
    );
    let user =
        ops::users::get_by_id(&state, user_id, params.target_domain.as_deref())
            .await?;
    Ok((StatusCode::OK, Json(user)))
}

/// GET /users/find?name=...&domain=...
pub async fn get_by_name_and_domain(
    State(state): State<AppState>,
    Query(params): Query<GetUserByNameDomainQuery>,
) -> Result<impl IntoResponse, ApiError> {
    info!(
        "GET /users/find?name={}&domain={}",
        params.name, params.domain
    );
    let user =
        ops::users::get_by_name_and_domain(&state, params.name, params.domain)
            .await?;
    Ok((StatusCode::OK, Json(user)))
}

/// GET /users/{user_id}/domains
pub async fn get_user_associated_domains(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<UserQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    info!(
        "GET /users/{user_id}/domains?target_domain={:?}",
        params.target_domain
    );
    let domains = ops::hosts::get_user_associated_domains(
        &state,
        user_id,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(domains)))
}

/// DELETE /users/{user_id}
pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    info!("DELETE /users/{user_id}");
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::users::auth::delete(),
    )
    .await?;
    ops::users::delete_home_user(&state, &session, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// DELETE /federation/users/{user_id}
    pub async fn delete(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path(user_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, ApiError> {
        info!("DELETE /federation/users/{user_id}");
        let session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::users::auth::federated::delete(),
        )
        .await?;
        ops::users::delete_remote_user_record(&state, &session, user_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
