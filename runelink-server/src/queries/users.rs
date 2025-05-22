use crate::{db::DbPool, error::ApiError};
use runelink_types::{NewUser, User};
use uuid::Uuid;

pub async fn insert_user(
    pool: &DbPool,
    new_user: &NewUser,
) -> Result<User, ApiError> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, domain)
        VALUES ($1, $2)
        RETURNING *;
        "#,
        new_user.name,
        new_user.domain,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_all_users(pool: &DbPool) -> Result<Vec<User>, ApiError> {
    sqlx::query_as!(User, "SELECT * FROM users;")
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)
}

pub async fn get_user_by_id(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<User, ApiError> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1;",
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}
