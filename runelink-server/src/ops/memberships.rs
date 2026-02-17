use runelink_client::{requests, util::get_api_url};
use runelink_types::{
    FullServerMembership, NewServerMembership, ServerMember, ServerMembership,
    UserRef,
};
use uuid::Uuid;

use crate::{
    auth::Session,
    error::{ApiError, ApiResult},
    queries,
    state::AppState,
};

/// Create a new membership for a user in a server.
pub async fn create(
    state: &AppState,
    session: &mut Session,
    new_membership: &NewServerMembership,
) -> ApiResult<FullServerMembership> {
    // If this membership is for a remote server, proxy via federation and cache locally.
    if state
        .config
        .is_remote_domain(Some(&new_membership.server_domain))
    {
        // Home Server should only create memberships for its own users.
        let user_domain = new_membership.user_ref.domain.clone();
        if state.config.is_remote_domain(Some(&user_domain)) {
            return Err(ApiError::BadRequest(
                "User domain in membership does not match local domain".into(),
            ));
        }
        let server_api_url = get_api_url(&new_membership.server_domain);
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            server_api_url.clone(),
            new_membership.user_ref.clone(),
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
    if new_membership.user_ref.domain != state.config.local_domain() {
        let user = session.lookup_user(state).await?;
        if user.is_none() {
            let api_url = get_api_url(&new_membership.user_ref.domain);
            let user = requests::users::fetch_by_ref(
                &state.http_client,
                &api_url,
                new_membership.user_ref.clone(),
            )
            .await?;
            queries::users::upsert_remote(&state.db_pool, &user).await?;
        }
    }

    // Create the membership
    let member =
        queries::memberships::insert_local(&state.db_pool, new_membership)
            .await?;
    let membership = queries::memberships::get_local_by_user_and_server(
        state,
        new_membership.server_id,
        new_membership.user_ref.clone(),
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
    target_domain: Option<&str>,
) -> ApiResult<Vec<ServerMember>> {
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
pub async fn get_member_by_user_and_server(
    state: &AppState,
    server_id: Uuid,
    user_ref: UserRef,
    target_domain: Option<&str>,
) -> ApiResult<ServerMember> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let member = queries::memberships::get_local_member_by_user_and_server(
            &state.db_pool,
            server_id,
            user_ref,
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
            user_ref,
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
    user_ref: UserRef,
) -> ApiResult<Vec<ServerMembership>> {
    let memberships =
        queries::memberships::get_by_user(state, user_ref).await?;
    Ok(memberships)
}

/// Delete a server membership.
pub async fn delete(
    state: &AppState,
    session: &mut Session,
    server_id: Uuid,
    user_ref: UserRef,
    target_domain: Option<&str>,
) -> ApiResult<()> {
    let session_user_ref = session.user_ref.clone().ok_or_else(|| {
        ApiError::AuthError("User reference required for leaving server".into())
    })?;
    if session_user_ref != user_ref {
        return Err(ApiError::BadRequest(
            "User identity in path does not match authenticated user".into(),
        ));
    }

    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        // Verify the membership exists
        queries::memberships::get_local_member_by_user_and_server(
            &state.db_pool,
            server_id,
            user_ref.clone(),
        )
        .await?;
        queries::memberships::delete_local(&state.db_pool, server_id, user_ref)
            .await?;
        Ok(())
    } else {
        // Delete on remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.clone(),
        )?;
        requests::memberships::federated::delete(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            user_ref.clone(),
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to leave server on {domain}: {e}"
            ))
        })?;
        // Also delete from local cache if it exists
        let _ = queries::memberships::delete_remote(
            &state.db_pool,
            server_id,
            user_ref,
        )
        .await;
        Ok(())
    }
}

/// Auth requirements for membership operations.
pub mod auth {
    use super::*;
    use crate::auth::Requirement as Req;
    use crate::or;

    pub fn create(_server_id: Uuid) -> Req {
        // TODO: make this admin only and create an invite system
        // Servers should also be public or private
        Req::Always.or_admin().client_only()
    }

    pub fn delete(server_id: Uuid, user_ref: UserRef) -> Req {
        or!(Req::User(user_ref), Req::ServerAdmin(server_id))
            .or_admin()
            .client_only()
    }

    pub mod federated {
        use super::*;

        pub fn create(_server_id: Uuid, user_ref: UserRef) -> Req {
            Req::FederatedUser(user_ref).federated_only()
        }

        pub fn delete(_server_id: Uuid, user_ref: UserRef) -> Req {
            Req::FederatedUser(user_ref).federated_only()
        }
    }
}
