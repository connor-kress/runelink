use log::warn;
use runelink_client::{requests, util::get_api_url};
use runelink_types::{NewUser, User};
use uuid::Uuid;

use crate::{
    auth::Session,
    error::{ApiError, ApiResult},
    queries,
    state::AppState,
};

/// Create a new user.
pub async fn create(
    state: &AppState,
    _session: &Session,
    new_user: &NewUser,
) -> ApiResult<User> {
    let user = queries::users::insert(&state.db_pool, new_user).await?;
    Ok(user)
}

/// List all users (public).
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local users.
pub async fn get_all(
    state: &AppState,
    target_domain: Option<&str>,
) -> ApiResult<Vec<User>> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let users = queries::users::get_all(&state.db_pool).await?;
        Ok(users)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let users =
            requests::users::fetch_all(&state.http_client, &api_url, None)
                .await
                .map_err(|e| {
                    ApiError::Internal(format!(
                        "Failed to fetch users from {domain}: {e}"
                    ))
                })?;
        Ok(users)
    }
}

/// Get a user by ID (public).
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local user.
pub async fn get_by_id(
    state: &AppState,
    user_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<User> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let user = queries::users::get_by_id(&state.db_pool, user_id).await?;
        Ok(user)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user = requests::users::fetch_by_id(
            &state.http_client,
            &api_url,
            user_id,
            None,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch user from {domain}: {e}"
            ))
        })?;
        Ok(user)
    }
}

/// Find a user by name and domain (public).
/// The domain parameter specifies which domain to query (local or remote).
pub async fn get_by_name_and_domain(
    state: &AppState,
    name: String,
    domain: String,
) -> ApiResult<User> {
    // Handle local case
    if domain == state.config.local_domain().as_str() {
        let user = queries::users::get_by_name_and_domain(
            &state.db_pool,
            name,
            domain,
        )
        .await?;
        Ok(user)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let api_url = get_api_url(domain.as_str());
        let user = requests::users::fetch_by_name_and_domain(
            &state.http_client,
            &api_url,
            name,
            domain.clone(),
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch user from {domain}: {e}"
            ))
        })?;
        Ok(user)
    }
}

/// Delete a user from their home server.
/// This will additionally send federated delete requests to all foreign
/// servers where the user has memberships (best-effort).
pub async fn delete_home_user(
    state: &AppState,
    _session: &Session,
    user_id: Uuid,
) -> ApiResult<()> {
    // Load user and verify they belong to the local domain
    let user = queries::users::get_by_id(&state.db_pool, user_id).await?;
    if user.domain != state.config.local_domain() {
        return Err(ApiError::BadRequest(
            "Can only delete users from their home server".into(),
        ));
    }

    // Get all foreign server domains where this user has memberships
    let foreign_domains =
        queries::memberships::get_remote_server_domains_for_user(
            &state.db_pool,
            user_id,
        )
        .await?;

    // Send federated delete requests to each foreign server (best-effort)
    for domain in &foreign_domains {
        let api_url = get_api_url(domain);
        let token_result = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_id,
            user.domain.clone(),
        );
        match token_result {
            Ok(token) => {
                let user_result = requests::users::federated::delete(
                    &state.http_client,
                    &api_url,
                    &token,
                    user_id,
                )
                .await;
                if let Err(e) = user_result {
                    warn!(
                        "Failed to delete user {user_id} on foreign server {domain}: {e}"
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to issue federation token for user {user_id} on domain {domain}: {e}"
                );
            }
        }
    }

    // Delete the user record (cascades will handle local_accounts, memberships, etc.)
    queries::users::delete_by_id(&state.db_pool, user_id).await?;
    Ok(())
}

/// Delete a remote user record from a foreign server.
/// This is called by the home server via federation to delete a cached user record.
pub async fn delete_remote_user_record(
    state: &AppState,
    session: &Session,
    user_id: Uuid,
) -> ApiResult<()> {
    // Require user_ref exists and matches the user_id
    let user_ref = session.user_ref.as_ref().ok_or_else(|| {
        ApiError::AuthError(
            "User reference required for federated user deletion".into(),
        )
    })?;
    if user_ref.id != user_id {
        return Err(ApiError::BadRequest(
            "User ID in path does not match user reference in token".into(),
        ));
    }
    // Require the user is not from the local domain
    if user_ref.domain == state.config.local_domain() {
        return Err(ApiError::BadRequest(
            "Cannot delete local users via federation".into(),
        ));
    }

    // Verify the caller is the home server for this user
    let expected_home_server_url = get_api_url(&user_ref.domain);
    let federation_claims = session.federation.as_ref().ok_or_else(|| {
        ApiError::AuthError("Federation claims required".into())
    })?;

    if federation_claims.iss != expected_home_server_url {
        return Err(ApiError::AuthError(
            "Only the home server can delete a user".into(),
        ));
    }

    // Delete the remote user record
    queries::users::delete_by_id_and_domain(
        &state.db_pool,
        user_id,
        &user_ref.domain,
    )
    .await?;
    Ok(())
}

/// Auth requirements for user operations.
pub mod auth {
    use crate::auth::Requirement as Req;

    pub fn create() -> Req {
        Req::And(vec![Req::Client, Req::HostAdmin])
    }

    pub fn delete() -> Req {
        Req::And(vec![Req::Client, Req::HostAdmin])
    }

    pub mod federated {
        use super::*;

        pub fn delete() -> Req {
            Req::And(vec![Req::Federation])
        }
    }
}
