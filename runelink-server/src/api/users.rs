use crate::{error::ApiError, ops, state::AppState};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
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
pub async fn create_user(
    State(state): State<AppState>,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    let user = ops::create_user(&state, &new_user).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// GET /users
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let users = ops::list_users(&state).await?;
    Ok((StatusCode::OK, Json(users)))
}

/// GET /users/{user_id}
pub async fn get_user_by_id_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let user = ops::get_user_by_id(&state, user_id).await?;
    Ok((StatusCode::OK, Json(user)))
}

/// GET /users/find?name=...&domain=...
pub async fn find_user_by_name_domain_handler(
    State(state): State<AppState>,
    Query(params): Query<GetUserByNameDomainQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let user =
        ops::find_user_by_name_domain(&state, params.name, params.domain)
            .await?;
    Ok((StatusCode::OK, Json(user)))
}

/// GET /users/{user_id}/domains
pub async fn get_user_associated_domains(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let domains = ops::get_user_associated_domains(&state, user_id).await?;
    Ok((StatusCode::OK, Json(domains)))
}
