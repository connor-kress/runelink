use super::Session;
use crate::auth::Requirement;
use crate::{auth::AuthSpec, error::ApiError, queries, state::AppState};
use runelink_client::{requests, util::get_api_url};
use runelink_types::{
    FullServerMembership, NewServerMembership, ServerMember, ServerMembership,
};
use uuid::Uuid;

/// Auth requirements for `create_remote_membership` (federation).
pub fn auth_federation_create_membership(_server_id: Uuid) -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::Federation {
            scopes: vec!["write:memberships"],
        }],
    }
}

/// Auth requirements for `add_server_member`.
pub fn auth_add_server_member(_server_id: Uuid) -> AuthSpec {
    AuthSpec {
        // TODO: make this admin only and create an invite system
        requirements: vec![],
    }
}

/// Add a user to a server (handles both local and remote users).
pub async fn add_server_member(
    state: &AppState,
    _session: &Session,
    server_id: Uuid,
    new_membership: &NewServerMembership,
) -> Result<FullServerMembership, ApiError> {
    let member;
    if new_membership.user_domain == state.config.local_domain() {
        // Local user (just add directly)
        member = queries::add_user_to_server(
            &state.db_pool,
            server_id,
            new_membership,
        )
        .await?;
    } else {
        // Remote user (handle syncing with user's home server)
        let api_url = get_api_url(&new_membership.user_domain);
        let remote_user =
            queries::get_user_by_id(&state.db_pool, new_membership.user_id)
                .await;
        let _user = match remote_user {
            Err(ApiError::NotFound) => {
                // Remote user is not in the local database
                let user = requests::fetch_user_by_id(
                    &state.http_client,
                    &api_url,
                    new_membership.user_id,
                )
                .await?;
                queries::insert_remote_user(&state.db_pool, &user).await?
            }
            other => other?,
        };
        member = queries::add_user_to_server(
            &state.db_pool,
            server_id,
            new_membership,
        )
        .await?;
    }
    let membership = queries::get_local_server_membership(
        state,
        server_id,
        new_membership.user_id,
    )
    .await?;
    let full_membership = FullServerMembership {
        server: membership.server,
        user: member.user,
        role: membership.role,
        joined_at: membership.joined_at,
        updated_at: membership.updated_at,
        synced_at: membership.synced_at,
    };
    Ok(full_membership)
}

/// List all members of a server (public).
pub async fn list_server_members(
    state: &AppState,
    server_id: Uuid,
) -> Result<Vec<ServerMember>, ApiError> {
    let members =
        queries::get_all_server_members(&state.db_pool, server_id).await?;
    Ok(members)
}

/// Get a specific server member (public).
pub async fn get_server_member(
    state: &AppState,
    server_id: Uuid,
    user_id: Uuid,
) -> Result<ServerMember, ApiError> {
    let member =
        queries::get_server_member(&state.db_pool, server_id, user_id).await?;
    Ok(member)
}

/// Add a user to a remote server (federation endpoint).
pub async fn add_remote_server_member(
    state: &AppState,
    server_id: Uuid,
    membership: &ServerMembership,
) -> Result<ServerMembership, ApiError> {
    // Ensure path and payload server ID match
    if membership.server.id != server_id {
        return Err(ApiError::Unknown(
            "Path server ID doesn't match payload server ID.".into(),
        ));
    }
    // Upsert the remote server into cached_remote_servers
    queries::upsert_cached_remote_server(&state.db_pool, &membership.server)
        .await?;
    // Insert the membership itself
    let new_membership = queries::insert_user_remote_server_membership(
        &state.db_pool,
        membership,
    )
    .await?;
    Ok(new_membership)
}
