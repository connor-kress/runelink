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
pub async fn get_all(state: &AppState) -> Result<Vec<User>, ApiError> {
    let users = queries::users::get_all(&state.db_pool).await?;
    Ok(users)
}

/// Get a user by ID (public).
pub async fn get_by_id(
    state: &AppState,
    user_id: Uuid,
) -> Result<User, ApiError> {
    let user = queries::users::get_by_id(&state.db_pool, user_id).await?;
    Ok(user)
}

/// Find a user by name and domain (public).
pub async fn get_by_name_and_domain(
    state: &AppState,
    name: String,
    domain: String,
) -> Result<User, ApiError> {
    let user =
        queries::users::get_by_name_and_domain(&state.db_pool, name, domain)
            .await?;
    Ok(user)
}

/// Auth requirements for user operations.
pub mod auth {
    use super::*;

    pub fn create() -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::HostAdmin],
        }
    }

    pub mod federated {
        use super::*;

        pub fn get_by_id() -> AuthSpec {
            AuthSpec {
                requirements: vec![Requirement::Federation],
            }
        }
    }
}
