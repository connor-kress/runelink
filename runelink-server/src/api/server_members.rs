use crate::{
    auth::{Principal, authorize},
    error::ApiError,
    ops,
    state::AppState,
};
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use runelink_client::{requests, util::get_api_url};
use runelink_types::NewServerMembership;
use uuid::Uuid;

/// GET /servers/{server_id}/users
pub async fn list_server_members(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let members =
        ops::memberships::get_members_by_server(&state, server_id).await?;
    Ok((StatusCode::OK, Json(members)))
}

/// GET /servers/{server_id}/users/{user_id}
pub async fn get_server_member(
    State(state): State<AppState>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let member = ops::memberships::get_member_by_user_and_server(
        &state, server_id, user_id,
    )
    .await?;
    Ok((StatusCode::OK, Json(member)))
}

/// POST /servers/{server_id}/users
pub async fn add_server_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<Uuid>,
    Json(new_membership): Json<NewServerMembership>,
) -> Result<impl IntoResponse, ApiError> {
    if server_id != new_membership.server_id {
        return Err(ApiError::BadRequest(
            "Server ID in path does not match server ID in membership".into(),
        ));
    }
    if new_membership.user_domain != state.config.local_domain() {
        return Err(ApiError::BadRequest(
            "User domain in membership does not match local domain".into(),
        ));
    }
    let mut session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::memberships::auth::create(),
    )
    .await?;
    if new_membership.server_domain == state.config.local_domain() {
        let full_membership = ops::memberships::create(
            &state,
            &mut session,
            server_id,
            &new_membership,
        )
        .await?;
        Ok((StatusCode::CREATED, Json(full_membership)))
    } else {
        let server_api_url = get_api_url(&new_membership.server_domain);
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            server_api_url.clone(),
            new_membership.user_id,
            new_membership.user_domain.clone(),
        )?;
        let full_membership = requests::memberships::federated::create(
            &state.http_client,
            &server_api_url,
            &token,
            server_id,
            &new_membership,
        )
        .await?;
        // We will use this membership as it has the correct synced_at timestamp
        let membership = ops::memberships::add_remote(
            &state,
            server_id,
            &full_membership.clone().into(),
        )
        .await?;
        Ok((
            StatusCode::CREATED,
            Json(membership.as_full(full_membership.user)),
        ))
    }
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// POST /federation/servers/{server_id}/users
    ///
    /// Create a remote membership (requires federation auth).
    pub async fn add_server_member(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path(server_id): Path<Uuid>,
        Json(new_membership): Json<NewServerMembership>,
    ) -> Result<impl IntoResponse, ApiError> {
        if server_id != new_membership.server_id {
            return Err(ApiError::BadRequest(
                "Server ID in path does not match server ID in membership"
                    .into(),
            ));
        }
        if new_membership.user_domain == state.config.local_domain() {
            return Err(ApiError::BadRequest(
                "User domain in membership should not match local domain"
                    .into(),
            ));
        }
        let mut session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::memberships::auth::federated::create(),
        )
        .await?;
        let full_membership = ops::memberships::create(
            &state,
            &mut session,
            server_id,
            &new_membership,
        )
        .await?;
        Ok((StatusCode::CREATED, Json(full_membership)))
    }
}
