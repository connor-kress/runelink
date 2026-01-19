use runelink_client::{requests::users, util::get_api_url};
use runelink_types::{NewUser, User};
use uuid::Uuid;

use crate::{
    auth::{AuthSpec, Requirement, Session},
    error::ApiError,
    queries,
    state::AppState,
};

/// Create a new user.
pub async fn create(
    state: &AppState,
    _session: &Session,
    new_user: &NewUser,
) -> Result<User, ApiError> {
    let user = queries::users::insert(&state.db_pool, new_user).await?;
    Ok(user)
}

/// List all users (public).
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local users.
pub async fn get_all(
    state: &AppState,
    target_domain: Option<&str>,
) -> Result<Vec<User>, ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        let users = queries::users::get_all(&state.db_pool).await?;
        Ok(users)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let users =
            users::fetch_all(&state.http_client, &api_url, Some(domain))
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
) -> Result<User, ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        let user = queries::users::get_by_id(&state.db_pool, user_id).await?;
        Ok(user)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user = users::fetch_by_id(
            &state.http_client,
            &api_url,
            user_id,
            target_domain,
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
) -> Result<User, ApiError> {
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
        let user = users::fetch_by_name_and_domain(
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

/// Auth requirements for user operations.
pub mod auth {
    use super::*;

    pub fn create() -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::HostAdmin],
        }
    }
}
