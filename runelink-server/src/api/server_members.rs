use crate::{error::ApiError, queries, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_client::{requests, util::get_api_url};
use runelink_types::{NewServerMember, ServerMembership};
use uuid::Uuid;

/// POST /servers/{server_id}/users
pub async fn add_server_member(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Json(new_member): Json<NewServerMember>,
) -> Result<impl IntoResponse, ApiError> {
    let member;
    if new_member.user_domain == state.config.local_domain_with_port() {
        // Local user (just add directly)
        member = queries::add_user_to_server(
            &state.db_pool,
            server_id,
            &new_member,
        ).await?;
    } else {
        // Remote user (handle syncing with user's home server)
        let api_url = get_api_url(&new_member.user_domain);
        let remote_user =
            queries::get_user_by_id(&state.db_pool, new_member.user_id).await;
        if matches!(remote_user, Err(ApiError::NotFound)) {
            // Remote user is not in the local database
            let user = requests::fetch_user_by_id(
                &state.http_client, &api_url, new_member.user_id
            ).await?;
            queries::insert_remote_user(&state.db_pool, &user).await?;
        } else {
            // Raise other potential errors
            remote_user?;
        }
        member = queries::add_user_to_server(
            &state.db_pool, server_id, &new_member
        ).await?;
        let membership = queries::get_local_server_membership(
            &state, server_id, new_member.user_id
        ).await?;
        requests::sync_remote_membership(
            &state.http_client, &api_url, &membership
        ).await?;
        // TODO: remove membership if sync failed
    }
    Ok((StatusCode::CREATED, Json(member)))
}

/// GET /servers/{server_id}/users
pub async fn list_server_members(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_server_members(&state.db_pool, server_id).await.map(Json)
}

/// GET /servers/{server_id}/users/{user_id}
pub async fn get_server_member(
    State(state): State<AppState>,
    Path((server_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_server_member(&state.db_pool, server_id, user_id)
        .await
        .map(Json)
}

/// POST /servers/{server_id}/remote-memberships
pub async fn create_remote_membership(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Json(membership): Json<ServerMembership>,
) -> Result<impl IntoResponse, ApiError> {
    // Ensure path and payload server ID match
    if membership.server.id != server_id {
        return Err(ApiError::Unknown(
            "Path server ID doesn't match payoad server ID.".into(),
        ));
    }
    // Upsert the remote‐server into cached_remote_servers
    queries::upsert_cached_remote_server(
        &state.db_pool,
        &membership.server
    ).await?;
    // Insert the membership itself
    let new_membership = queries::insert_user_remote_server_membership(
        &state.db_pool,
        &membership,
    ).await?;
    Ok((StatusCode::CREATED, Json(new_membership)))
}
