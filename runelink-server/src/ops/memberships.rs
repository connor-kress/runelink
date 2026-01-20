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
    new_membership: &NewServerMembership,
) -> Result<FullServerMembership, ApiError> {
    // If this membership is for a remote server, proxy via federation and cache locally.
    if state
        .config
        .is_remote_domain(Some(&new_membership.server_domain))
    {
        // Home Server should only create memberships for its own users.
        if new_membership.user_domain != state.config.local_domain() {
            return Err(ApiError::BadRequest(
                "User domain in membership does not match local domain".into(),
            ));
        }
        let server_api_url = get_api_url(&new_membership.server_domain);
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            server_api_url.clone(),
            new_membership.user_id,
            new_membership.user_domain.clone(),
        )?;
        let membership = requests::memberships::federated::create(
            &state.http_client,
            &server_api_url,
            &token,
            new_membership,
        )
        .await?;
        let user = membership.user.clone();
        // Cache the remote server and membership locally
        queries::servers::upsert_remote(&state.db_pool, &membership.server)
            .await?;
        let cached_membership = queries::memberships::insert_remote(
            &state.db_pool,
            &membership.into(),
        )
        .await?;
        // synced_at comes from cached membership
        return Ok(cached_membership.as_full(user));
    }

    // Ensure remote user exists locally before creating membership
    if new_membership.user_domain != state.config.local_domain() {
        let user = session.lookup_user(state).await?;
        if user.is_none() {
            let api_url = get_api_url(&new_membership.user_domain);
            let user = requests::users::fetch_by_id(
                &state.http_client,
                &api_url,
                new_membership.user_id,
                None,
            )
            .await?;
            queries::users::insert_remote(&state.db_pool, &user).await?;
        }
    }

    // Create the membership
    let member =
        queries::memberships::insert_local(&state.db_pool, new_membership)
            .await?;
    let membership = queries::memberships::get_local_by_user_and_server(
        state,
        new_membership.server_id,
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
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local members.
pub async fn get_members_by_server(
    state: &AppState,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> Result<Vec<ServerMember>, ApiError> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let members = queries::memberships::get_members_by_server(
            &state.db_pool,
            server_id,
        )
        .await?;
        Ok(members)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let members = requests::memberships::fetch_members_by_server(
            &state.http_client,
            &api_url,
            server_id,
            None,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch members from {domain}: {e}"
            ))
        })?;
        Ok(members)
    }
}

/// Get a specific server member (public).
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local member.
pub async fn get_member_by_user_and_server(
    state: &AppState,
    server_id: Uuid,
    user_id: Uuid,
    target_domain: Option<&str>,
) -> Result<ServerMember, ApiError> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let member = queries::memberships::get_local_member_by_user_and_server(
            &state.db_pool,
            server_id,
            user_id,
        )
        .await?;
        Ok(member)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let member = requests::memberships::fetch_member_by_user_and_server(
            &state.http_client,
            &api_url,
            server_id,
            user_id,
            None,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch member from {domain}: {e}"
            ))
        })?;
        Ok(member)
    }
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
