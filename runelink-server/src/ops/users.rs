use crate::{error::ApiError, queries, state::AppState};
use runelink_types::NewUser;
use uuid::Uuid;

/// Create a new user.
pub async fn create_user(
    state: &AppState,
    new_user: &NewUser,
) -> Result<runelink_types::User, ApiError> {
    queries::insert_user(&state.db_pool, new_user).await
}

/// List all users.
pub async fn list_users(
    state: &AppState,
) -> Result<Vec<runelink_types::User>, ApiError> {
    queries::get_all_users(&state.db_pool).await
}

/// Get a user by ID.
pub async fn get_user_by_id(
    state: &AppState,
    user_id: Uuid,
) -> Result<runelink_types::User, ApiError> {
    queries::get_user_by_id(&state.db_pool, user_id).await
}

/// Find a user by name and domain.
pub async fn find_user_by_name_domain(
    state: &AppState,
    name: String,
    domain: String,
) -> Result<runelink_types::User, ApiError> {
    queries::get_user_by_name_and_domain(&state.db_pool, name, domain).await
}

/// Get all domains associated with a user.
pub async fn get_user_associated_domains(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<String>, ApiError> {
    queries::get_associated_domains_for_user(&state.db_pool, user_id).await
}
