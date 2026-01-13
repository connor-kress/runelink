use super::Session;
use crate::{
    auth::{AuthSpec, Requirement},
    error::ApiError,
    queries,
    state::AppState,
};
use runelink_types::{NewUser, User};
use uuid::Uuid;

/// Auth requirements for `create_user`.
pub fn auth_create_user() -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::HostAdmin],
    }
}

/// Create a new user.
pub async fn create_user(
    state: &AppState,
    _session: &Session,
    new_user: &NewUser,
) -> Result<User, ApiError> {
    let user = queries::users::insert(&state.db_pool, new_user).await?;
    Ok(user)
}

/// List all users (public).
pub async fn list_users(state: &AppState) -> Result<Vec<User>, ApiError> {
    let users = queries::users::get_all(&state.db_pool).await?;
    Ok(users)
}

/// Auth requirements for `get_user_by_id` (federation).
pub fn auth_federation_get_user() -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::Federation],
    }
}

/// Get a user by ID (public).
pub async fn get_user_by_id(
    state: &AppState,
    user_id: Uuid,
) -> Result<User, ApiError> {
    let user = queries::users::get_by_id(&state.db_pool, user_id).await?;
    Ok(user)
}

/// Find a user by name and domain (public).
pub async fn find_user_by_name_domain(
    state: &AppState,
    name: String,
    domain: String,
) -> Result<User, ApiError> {
    let user =
        queries::users::get_by_name_and_domain(&state.db_pool, name, domain)
            .await?;
    Ok(user)
}

/// Get all domains associated with a user (public).
pub async fn get_user_associated_domains(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<String>, ApiError> {
    let domains =
        queries::hosts::get_user_associated_domains(&state.db_pool, user_id)
            .await?;
    Ok(domains)
}
