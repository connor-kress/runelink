use crate::{error::ApiError, queries, state::AppState};
use runelink_client::{requests, util::get_api_url};
use runelink_types::{NewServerMember, ServerMembership};
use uuid::Uuid;

/// Add a user to a server (handles both local and remote users).
pub async fn add_server_member(
    state: &AppState,
    server_id: Uuid,
    new_member: &NewServerMember,
) -> Result<runelink_types::ServerMember, ApiError> {
    if new_member.user_domain == state.config.local_domain_with_port() {
        // Local user (just add directly)
        queries::add_user_to_server(&state.db_pool, server_id, new_member).await
    } else {
        // Remote user (handle syncing with user's home server)
        let api_url = get_api_url(&new_member.user_domain);
        let remote_user =
            queries::get_user_by_id(&state.db_pool, new_member.user_id).await;
        if matches!(remote_user, Err(ApiError::NotFound)) {
            // Remote user is not in the local database
            let user = requests::fetch_user_by_id(
                &state.http_client,
                &api_url,
                new_member.user_id,
            )
            .await?;
            queries::insert_remote_user(&state.db_pool, &user).await?;
        } else {
            // Raise other potential errors
            remote_user?;
        }
        let member =
            queries::add_user_to_server(&state.db_pool, server_id, new_member)
                .await?;
        let membership = queries::get_local_server_membership(
            state,
            server_id,
            new_member.user_id,
        )
        .await?;
        requests::sync_remote_membership(
            &state.http_client,
            &api_url,
            &membership,
        )
        .await?;
        // TODO: remove membership if sync failed
        Ok(member)
    }
}

/// List all members of a server.
pub async fn list_server_members(
    state: &AppState,
    server_id: Uuid,
) -> Result<Vec<runelink_types::ServerMember>, ApiError> {
    queries::get_all_server_members(&state.db_pool, server_id).await
}

/// Get a specific server member.
pub async fn get_server_member(
    state: &AppState,
    server_id: Uuid,
    user_id: Uuid,
) -> Result<runelink_types::ServerMember, ApiError> {
    queries::get_server_member(&state.db_pool, server_id, user_id).await
}

/// Create a remote membership (federation endpoint).
pub async fn create_remote_membership(
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
    queries::insert_user_remote_server_membership(&state.db_pool, membership)
        .await
}
