use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::{Message, User};
use sqlx::types::Json;
use uuid::Uuid;

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
