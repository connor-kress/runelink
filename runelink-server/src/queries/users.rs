use crate::{db::DbPool, error::ApiError};
use runelink_types::{NewUser, User};
use time::OffsetDateTime;
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

pub async fn insert_remote_user(
    pool: &DbPool,
    remote_user: &User,
) -> Result<User, ApiError> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, name, domain, created_at, updated_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *;
        "#,
        remote_user.id,
        remote_user.name,
        remote_user.domain,
        remote_user.created_at,
        remote_user.updated_at,
        OffsetDateTime::now_utc(),
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
        SELECT DISTINCT s.domain
        FROM user_remote_server_memberships m
        JOIN cached_remote_servers s ON s.id = m.remote_server_id
        WHERE m.user_id = $1
        ORDER BY s.domain ASC;
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}
