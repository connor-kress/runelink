use uuid::Uuid;
use sqlx::types::Json;

use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::{Channel, Host, Message, NewUser, Server, ServerWithChannels, User};

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

pub async fn get_all_messages(
    pool: &DbPool,
) -> Result<Vec<Message>, ApiError> {
    sqlx::query_as!(
        Message,
        r#"
        SELECT
            m.id,
            m.channel_id,
            m.body,
            m.created_at,
            m.updated_at,
            to_jsonb(a) AS "author: Json<User>"
        FROM messages m
        LEFT JOIN users a ON a.name = m.author_name
                         AND a.domain = m.author_domain
        ORDER BY m.created_at DESC;
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_message_by_id(
    pool: &DbPool,
    msg_id: Uuid,
) -> Result<Message, ApiError> {
    sqlx::query_as!(
        Message,
        r#"
        SELECT
            m.id,
            m.channel_id,
            m.body,
            m.created_at,
            m.updated_at,
            to_jsonb(a) AS "author: Json<User>"
        FROM messages m
        LEFT JOIN users a ON a.name = m.author_name
                         AND a.domain = m.author_domain
        WHERE m.id = $1;
        "#,
        msg_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_channel_by_id(
    pool: &DbPool,
    channel_id: Uuid,
) -> Result<Channel, ApiError> {
    sqlx::query_as!(
        Channel,
        "SELECT * FROM channels WHERE id = $1;",
        channel_id
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_server_with_channels(
    pool: &DbPool,
    server_id: Uuid,
) -> Result<ServerWithChannels, ApiError> {
    let server = sqlx::query_as!(
        Server,
        "SELECT * FROM servers WHERE id = $1;",
        server_id,
    )
    .fetch_one(pool)
    .await?;

    let channels = sqlx::query_as!(
        Channel,
        r#"
        SELECT * FROM channels
        WHERE server_id = $1
        ORDER BY created_at DESC;
        "#,
        server_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(ServerWithChannels { server, channels })
}

pub async fn get_all_servers(
    pool: &DbPool,
) -> Result<Vec<Server>, ApiError> {
    sqlx::query_as!(
        Server,
        "SELECT * FROM servers",
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_all_hosts(pool: &DbPool) -> Result<Vec<Host>, ApiError> {
    sqlx::query_as!(
        Host,
        "SELECT * FROM hosts ORDER BY user_count DESC;",
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_host_by_domain(
    pool: &DbPool,
    domain: &str,
) -> Result<Host, ApiError> {
    sqlx::query_as!(
        Host,
        "SELECT * FROM hosts WHERE domain = $1;",
        domain,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}
