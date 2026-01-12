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
    let user = queries::insert_user(&state.db_pool, new_user).await?;
    Ok(user)
}

/// List all users (public).
pub async fn list_users(state: &AppState) -> Result<Vec<User>, ApiError> {
    let users = queries::get_all_users(&state.db_pool).await?;
    Ok(users)
}

/// Auth requirements for `get_user_by_id` (federation).
pub fn auth_federation_get_user() -> AuthSpec {
    AuthSpec {
        requirements: vec![Requirement::Federation {
            scopes: vec!["read:users"],
        }],
    }
}

/// Get a user by ID (public).
pub async fn get_user_by_id(
    state: &AppState,
    user_id: Uuid,
) -> Result<User, ApiError> {
    let user = queries::get_user_by_id(&state.db_pool, user_id).await?;
    Ok(user)
}

/// Find a user by name and domain (public).
pub async fn find_user_by_name_domain(
    state: &AppState,
    name: String,
    domain: String,
) -> Result<User, ApiError> {
    let user =
        queries::get_user_by_name_and_domain(&state.db_pool, name, domain)
            .await?;
    Ok(user)
}

/// Get all domains associated with a user (public).
pub async fn get_user_associated_domains(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<String>, ApiError> {
    let domains =
        queries::get_associated_domains_for_user(&state.db_pool, user_id)
            .await?;
    Ok(domains)
}
