use crate::{db::DbPool, error::ApiError};
use runelink_types::{Message, NewMessage, User};
use sqlx::types::Json;
use uuid::Uuid;

pub async fn insert_message(
    pool: &DbPool,
    channel_id: Uuid,
    new_message: &NewMessage,
) -> Result<Message, ApiError> {
    let new_id: Uuid = sqlx::query_scalar!(
        r#"
        INSERT INTO messages (channel_id, author_id, body)
        VALUES ($1, $2, $3)
        RETURNING id;
        "#,
        channel_id,
        new_message.author_id,
        new_message.body,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    get_message_by_id(pool, new_id).await
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
        LEFT JOIN users a ON a.id = m.author_id
        ORDER BY m.created_at DESC;
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_messages_by_server(
    pool: &DbPool,
    server_id: Uuid,
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
        LEFT JOIN users a ON a.id = m.author_id
        JOIN channels c ON c.id = m.channel_id
        WHERE c.server_id = $1
        ORDER BY m.created_at DESC;
        "#,
        server_id,
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_messages_by_channel(
    pool: &DbPool,
    channel_id: Uuid,
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
        LEFT JOIN users a ON a.id = m.author_id
        WHERE m.channel_id = $1
        ORDER BY m.created_at DESC;
        "#,
        channel_id,
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
        LEFT JOIN users a ON a.id = m.author_id
        WHERE m.id = $1;
        "#,
        msg_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}
