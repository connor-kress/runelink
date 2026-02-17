use crate::{
    auth::{Principal, authorize},
    error::{ApiError, ApiResult},
    ops,
    state::AppState,
};
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use log::info;
use runelink_types::{NewServerMembership, UserRef};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct MembershipQueryParams {
    pub target_domain: Option<String>,
}

/// GET /servers/{server_id}/users
pub async fn get_members_by_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Query(params): Query<MembershipQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "GET /servers/{server_id}/users?target_domain={:?}",
        params.target_domain
    );
    let members = ops::memberships::get_members_by_server(
        &state,
        server_id,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(members)))
}

/// GET /servers/{server_id}/users/{domain}/{name}
pub async fn get_by_user_and_server(
    State(state): State<AppState>,
    Path((server_id, domain, name)): Path<(Uuid, String, String)>,
    Query(params): Query<MembershipQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "GET /servers/{server_id}/users/{domain}/{name}?target_domain={:?}",
        params.target_domain
    );
    let member = ops::memberships::get_member_by_user_and_server(
        &state,
        server_id,
        UserRef::new(name, domain),
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(member)))
}

/// POST /servers/{server_id}/users
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<Uuid>,
    Json(new_membership): Json<NewServerMembership>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "POST /servers/{server_id}/users\nnew_membership = {:#?}",
        new_membership
    );
    if server_id != new_membership.server_id {
        return Err(ApiError::BadRequest(
            "Server ID in path does not match server ID in membership".into(),
        ));
    }
    let mut session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::memberships::auth::create(server_id),
    )
    .await?;
    let membership =
        ops::memberships::create(&state, &mut session, &new_membership).await?;
    Ok((StatusCode::CREATED, Json(membership)))
}

/// GET /users/{domain}/{name}/memberships
pub async fn get_by_user(
    State(state): State<AppState>,
    Path((domain, name)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    info!("GET /users/{domain}/{name}/servers");
    let user_ref = UserRef::new(name, domain);
    let memberships = ops::memberships::get_by_user(&state, user_ref).await?;
    Ok((StatusCode::OK, Json(memberships)))
}

/// DELETE /servers/{server_id}/users/{domain}/{name}
pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server_id, domain, name)): Path<(Uuid, String, String)>,
    Query(params): Query<MembershipQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "DELETE /servers/{server_id}/users/{domain}/{name}?target_domain={:?}",
        params.target_domain
    );
    let user_ref = UserRef::new(name, domain);
    let mut session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::memberships::auth::delete(server_id, user_ref.clone()),
    )
    .await?;
    ops::memberships::delete(
        &state,
        &mut session,
        server_id,
        user_ref,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// POST /federation/servers/{server_id}/users
    pub async fn create(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path(server_id): Path<Uuid>,
        Json(new_membership): Json<NewServerMembership>,
    ) -> ApiResult<impl IntoResponse> {
        info!(
            "POST /federation/servers/{server_id}/users\nnew_membership = {:#?}",
            new_membership
        );
        if server_id != new_membership.server_id {
            return Err(ApiError::BadRequest(
                "Server ID in path does not match server ID in membership"
                    .into(),
            ));
        }
        if !state
            .config
            .is_remote_domain(Some(&new_membership.user_ref.domain))
        {
            return Err(ApiError::BadRequest(
                "User domain in membership should not match local domain"
                    .into(),
            ));
        }
        let mut session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::memberships::auth::federated::create(
                server_id,
                new_membership.user_ref.clone(),
            ),
        )
        .await?;
        let membership =
            ops::memberships::create(&state, &mut session, &new_membership)
                .await?;
        Ok((StatusCode::CREATED, Json(membership)))
    }

    /// DELETE /federation/servers/{server_id}/users/{domain}/{name}
    pub async fn delete(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path((server_id, domain, name)): Path<(Uuid, String, String)>,
    ) -> ApiResult<impl IntoResponse> {
        info!("DELETE /federation/servers/{server_id}/users/{domain}/{name}");
        let user_ref = UserRef::new(name, domain);
        let mut session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::memberships::auth::federated::delete(
                server_id,
                user_ref.clone(),
            ),
        )
        .await?;
        ops::memberships::delete(
            &state,
            &mut session,
            server_id,
            user_ref,
            None,
        )
        .await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
