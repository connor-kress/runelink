use log::warn;
use runelink_client::{requests, util::get_api_url};
use runelink_types::{NewUser, User, UserRef};

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
    if !state.config.is_remote_domain(target_domain) {
        let users = queries::users::get_all(&state.db_pool).await?;
        Ok(users)
    } else {
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

/// Find a user by UserRef (public).
pub async fn get_by_ref(
    state: &AppState,
    user_ref: UserRef,
    _target_domain: Option<&str>,
) -> ApiResult<User> {
    if !state.config.is_remote_domain(Some(&user_ref.domain)) {
        let user = queries::users::get_by_ref(&state.db_pool, user_ref).await?;
        Ok(user)
    } else {
        let api_url = get_api_url(&user_ref.domain);
        let domain = user_ref.domain.clone();
        let user = requests::users::fetch_by_ref(
            &state.http_client,
            &api_url,
            user_ref,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch user from {}: {e}",
                domain
            ))
        })?;
        Ok(user)
    }
}

/// Delete a user from their home server.
pub async fn delete_home_user(
    state: &AppState,
    _session: &Session,
    user_ref: &UserRef,
) -> ApiResult<()> {
    let user =
        queries::users::get_by_ref(&state.db_pool, user_ref.clone()).await?;
    if user.domain != state.config.local_domain() {
        return Err(ApiError::BadRequest(
            "Can only delete users from their home server".into(),
        ));
    }

    let foreign_domains =
        queries::memberships::get_remote_server_domains_for_user(
            &state.db_pool,
            user_ref.clone(),
        )
        .await?;

    for domain in &foreign_domains {
        let api_url = get_api_url(domain);
        let token_result = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.clone(),
        );
        match token_result {
            Ok(token) => {
                let user_result = requests::users::federated::delete(
                    &state.http_client,
                    &api_url,
                    &token,
                    user_ref.clone(),
                )
                .await;
                if let Err(e) = user_result {
                    warn!(
                        "Failed to delete user {user_ref} on foreign server {domain}: {e}"
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to issue federation token for user {user_ref} on domain {domain}: {e}"
                );
            }
        }
    }

    queries::users::delete(&state.db_pool, user_ref.clone()).await?;
    Ok(())
}

/// Delete a remote user record from a foreign server.
pub async fn delete_remote_user_record(
    state: &AppState,
    session: &Session,
    user_ref: &UserRef,
) -> ApiResult<()> {
    let session_user_ref = session.user_ref.clone().ok_or_else(|| {
        ApiError::AuthError(
            "User reference required for federated user deletion".into(),
        )
    })?;
    if session_user_ref.name != user_ref.name
        || session_user_ref.domain != user_ref.domain
    {
        return Err(ApiError::BadRequest(
            "User identity in path does not match user reference in token"
                .into(),
        ));
    }
    if session_user_ref.domain == state.config.local_domain() {
        return Err(ApiError::BadRequest(
            "Cannot delete local users via federation".into(),
        ));
    }

    let expected_home_server_url = get_api_url(&session_user_ref.domain);
    let federation_claims = session.federation.as_ref().ok_or_else(|| {
        ApiError::AuthError("Federation claims required".into())
    })?;

    if federation_claims.iss != expected_home_server_url {
        return Err(ApiError::AuthError(
            "Only the home server can delete a user".into(),
        ));
    }

    queries::users::delete(&state.db_pool, user_ref.clone()).await?;
    Ok(())
}

/// Auth requirements for user operations.
pub mod auth {
    use super::*;
    use crate::auth::Requirement as Req;

    pub fn create() -> Req {
        Req::Client
    }

    pub fn delete(user_ref: UserRef) -> Req {
        Req::User(user_ref).or_admin().client_only()
    }

    pub mod federated {
        use super::*;

        pub fn delete(user_ref: UserRef) -> Req {
            Req::FederatedUser(user_ref).federated_only()
        }
    }
}
