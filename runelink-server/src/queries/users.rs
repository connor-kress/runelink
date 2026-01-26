use runelink_types::{NewUser, User, UserRole};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{db::DbPool, error::ApiResult};

pub async fn insert(pool: &DbPool, new_user: &NewUser) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, domain, role)
        VALUES ($1, $2, $3)
        RETURNING
            id,
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

pub async fn insert_remote(
    pool: &DbPool,
    remote_user: &User,
) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, name, domain, role, created_at, updated_at, synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at;
        "#,
        remote_user.id,
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
            id,
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

pub async fn get_by_id(pool: &DbPool, user_id: Uuid) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at
        FROM users
        WHERE id = $1;
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_by_name_and_domain(
    pool: &DbPool,
    name: String,
    domain: String,
) -> ApiResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            name,
            domain,
            role AS "role: UserRole",
            created_at,
            updated_at,
            synced_at
        FROM users
        WHERE name = $1 AND domain = $2;
        "#,
        name,
        domain,
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn delete_by_id(pool: &DbPool, user_id: Uuid) -> ApiResult<()> {
    sqlx::query!("DELETE FROM users WHERE id = $1;", user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_by_id_and_domain(
    pool: &DbPool,
    user_id: Uuid,
    domain: &str,
) -> ApiResult<()> {
    sqlx::query!(
        "DELETE FROM users WHERE id = $1 AND domain = $2;",
        user_id,
        domain
    )
    .execute(pool)
    .await?;
    Ok(())
}
