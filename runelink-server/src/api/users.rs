use crate::{
    auth::{Principal, authorize},
    error::ApiResult,
    ops,
    state::AppState,
};
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use log::info;
use runelink_types::{NewUser, UserRef};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UserQueryParams {
    pub target_domain: Option<String>,
}

/// POST /users
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(new_user): Json<NewUser>,
) -> ApiResult<impl IntoResponse> {
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
) -> ApiResult<impl IntoResponse> {
    info!("GET /users?target_domain={:?}", params.target_domain);
    let users =
        ops::users::get_all(&state, params.target_domain.as_deref()).await?;
    Ok((StatusCode::OK, Json(users)))
}

/// GET /users/{domain}/{name}
pub async fn get_by_ref(
    State(state): State<AppState>,
    Path((domain, name)): Path<(String, String)>,
    Query(params): Query<UserQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "GET /users/{domain}/{name}?target_domain={:?}",
        params.target_domain
    );
    let user_ref = UserRef::new(name, domain);
    let user = ops::users::get_by_ref(
        &state,
        user_ref,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(user)))
}

/// GET /users/{domain}/{name}/domains
pub async fn get_user_associated_domains(
    State(state): State<AppState>,
    Path((domain, name)): Path<(String, String)>,
    Query(params): Query<UserQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "GET /users/{domain}/{name}/domains?target_domain={:?}",
        params.target_domain
    );
    let user_ref = UserRef::new(name, domain);
    let domains = ops::hosts::get_user_associated_domains(
        &state,
        user_ref,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(domains)))
}

/// DELETE /users/{domain}/{name}
pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((domain, name)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let user_ref = UserRef::new(name.clone(), domain.clone());
    info!("DELETE /users/{domain}/{name}");
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::users::auth::delete(user_ref.clone()),
    )
    .await?;
    ops::users::delete_home_user(&state, &session, &user_ref).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// DELETE /federation/users/{domain}/{name}
    pub async fn delete(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path((domain, name)): Path<(String, String)>,
    ) -> ApiResult<impl IntoResponse> {
        let user_ref = UserRef::new(name, domain);
        info!(
            "DELETE /federation/users/{}/{}",
            user_ref.domain, user_ref.name
        );
        let session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::users::auth::federated::delete(user_ref.clone()),
        )
        .await?;
        ops::users::delete_remote_user_record(&state, &session, &user_ref)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
