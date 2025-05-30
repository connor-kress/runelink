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

pub async fn get_user_by_name_and_domain(
    pool: &DbPool,
    name: String,
    domain: String,
) -> Result<User, ApiError> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE name = $1 AND domain = $2;",
        name,
        domain,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_associated_domains_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<String>, ApiError> {
    sqlx::query_scalar!(
        r#"
        SELECT domain
        FROM user_associated_domains
        WHERE user_id = $1
        ORDER BY domain ASC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn add_associated_domain_for_user(
    pool: &DbPool,
    user_id: Uuid,
    domain: &str,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO user_associated_domains (user_id, domain)
        VALUES ($1, $2)
        ON CONFLICT (user_id, domain) DO NOTHING
        "#,
        user_id,
        domain,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_associated_domain_for_user(
    pool: &DbPool,
    user_id: Uuid,
    domain: &str,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        DELETE FROM user_associated_domains
        WHERE user_id = $1 AND domain = $2
        "#,
        user_id,
        domain,
    )
    .execute(pool)
    .await?;
    Ok(())
}
