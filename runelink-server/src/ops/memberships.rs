use runelink_client::{requests, util::get_api_url};
use runelink_types::{
    FullServerMembership, NewServerMembership, ServerMember, ServerMembership,
};
use uuid::Uuid;

use crate::{
    auth::{AuthSpec, Requirement, Session},
    error::ApiError,
    queries,
    state::AppState,
};

/// Create a new membership for a user in a server (handles both local and remote users).
///
/// For remote users, this function will fetch and cache the user profile from their
/// home server if they don't already exist locally.
pub async fn create(
    state: &AppState,
    session: &mut Session,
    server_id: Uuid,
    new_membership: &NewServerMembership,
) -> Result<FullServerMembership, ApiError> {
    // Ensure remote user exists locally before creating membership
    if new_membership.user_domain != state.config.local_domain() {
        let user = session.lookup_user(state).await?;
        if user.is_none() {
            let api_url = get_api_url(&new_membership.user_domain);
            let user = requests::users::fetch_by_id(
                &state.http_client,
                &api_url,
                new_membership.user_id,
            )
            .await?;
            queries::users::insert_remote(&state.db_pool, &user).await?;
        }
    }

    // Create the membership
    let member =
        queries::memberships::insert(&state.db_pool, new_membership).await?;
    let membership = queries::memberships::get_local_by_user_and_server(
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

/// Get all members of a server (public).
pub async fn get_members_by_server(
    state: &AppState,
    server_id: Uuid,
) -> Result<Vec<ServerMember>, ApiError> {
    let members =
        queries::memberships::get_members_by_server(&state.db_pool, server_id)
            .await?;
    Ok(members)
}

/// Get a specific server member (public).
pub async fn get_member_by_user_and_server(
    state: &AppState,
    server_id: Uuid,
    user_id: Uuid,
) -> Result<ServerMember, ApiError> {
    let member = queries::memberships::get_member_by_user_and_server(
        &state.db_pool,
        server_id,
        user_id,
    )
    .await?;
    Ok(member)
}

/// Add a user to a remote server (federation endpoint).
pub async fn add_remote(
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
    queries::servers::upsert_remote(&state.db_pool, &membership.server).await?;
    // Insert the membership itself
    let new_membership =
        queries::memberships::insert_remote(&state.db_pool, membership).await?;
    Ok(new_membership)
}

/// Get all server memberships for a user (public).
pub async fn get_by_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>, ApiError> {
    let memberships = queries::memberships::get_by_user(state, user_id).await?;
    Ok(memberships)
}

/// Auth requirements for membership operations.
pub mod auth {
    use super::*;

    pub fn create() -> AuthSpec {
        AuthSpec {
            // TODO: make this admin only and create an invite system
            requirements: vec![],
        }
    }

    pub mod federated {
        use super::*;

        pub fn create() -> AuthSpec {
            AuthSpec {
                requirements: vec![Requirement::Federation],
            }
        }
    }
}
