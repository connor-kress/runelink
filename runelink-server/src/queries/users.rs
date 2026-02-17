use runelink_types::{NewUser, User, UserRef, UserRole};
use time::OffsetDateTime;

use crate::{db::DbPool, error::ApiResult};

pub async fn insert(pool: &DbPool, new_user: &NewUser) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, domain, role)
        VALUES ($1, $2, $3)
        RETURNING
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at;
        "#,
        new_user.name,
        new_user.domain,
        new_user.role as UserRole,
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn upsert_remote(
    pool: &DbPool,
    remote_user: &User,
) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, domain, role, created_at, updated_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (name, domain) DO UPDATE SET
            role = EXCLUDED.role,
            updated_at = EXCLUDED.updated_at,
            synced_at = EXCLUDED.synced_at
        RETURNING
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at;
        "#,
        remote_user.name,
        remote_user.domain,
        UserRole::User as UserRole,
        remote_user.created_at,
        remote_user.updated_at,
        OffsetDateTime::now_utc(),
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_all(pool: &DbPool) -> ApiResult<Vec<User>> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at
        FROM users;
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(users)
}

pub async fn ensure_exists(
    pool: &DbPool,
    user_ref: UserRef,
) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, domain, role)
        VALUES ($1, $2, 'user')
        ON CONFLICT (name, domain) DO UPDATE SET updated_at = NOW()
        RETURNING
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at;
        "#,
        user_ref.name,
        user_ref.domain,
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_by_ref(pool: &DbPool, user_ref: UserRef) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at
        FROM users
        WHERE name = $1 AND domain = $2;
        "#,
        user_ref.name,
        user_ref.domain,
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn delete(pool: &DbPool, user_ref: UserRef) -> ApiResult<()> {
    sqlx::query!(
        "DELETE FROM users WHERE name = $1 AND domain = $2;",
        user_ref.name,
        user_ref.domain
    )
    .execute(pool)
    .await?;
    Ok(())
}
