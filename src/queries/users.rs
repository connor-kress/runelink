use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::{NewUser, User};

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
