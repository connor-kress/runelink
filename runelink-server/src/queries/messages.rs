use runelink_types::{Message, NewMessage, User};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{db::DbPool, error::ApiResult};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author: Option<Json<User>>,
    pub body: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<DbMessage> for Message {
    fn from(msg: DbMessage) -> Self {
        Message {
            id: msg.id,
            channel_id: msg.channel_id,
            author: msg.author.map(|json_user| json_user.0),
            body: msg.body,
            created_at: msg.created_at,
            updated_at: msg.updated_at,
        }
    }
}

pub async fn insert(
    pool: &DbPool,
    channel_id: Uuid,
    new_message: &NewMessage,
) -> ApiResult<Message> {
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
    .await?;
    let message = get_by_id(pool, new_id).await?;
    Ok(message)
}

pub async fn get_all(pool: &DbPool) -> ApiResult<Vec<Message>> {
    let rows = sqlx::query_as!(
        DbMessage,
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
    .await?;
    let messages = rows.into_iter().map(Message::from).collect();
    Ok(messages)
}

pub async fn get_by_server(
    pool: &DbPool,
    server_id: Uuid,
) -> ApiResult<Vec<Message>> {
    let rows = sqlx::query_as!(
        DbMessage,
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
    .await?;
    let messages = rows.into_iter().map(Message::from).collect();
    Ok(messages)
}

pub async fn get_by_channel(
    pool: &DbPool,
    channel_id: Uuid,
) -> ApiResult<Vec<Message>> {
    let rows = sqlx::query_as!(
        DbMessage,
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
    .await?;
    let messages = rows.into_iter().map(Message::from).collect();
    Ok(messages)
}

pub async fn get_by_id(pool: &DbPool, msg_id: Uuid) -> ApiResult<Message> {
    let db_message = sqlx::query_as!(
        DbMessage,
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
    .await?;
    Ok(db_message.into())
}

pub async fn delete(pool: &DbPool, message_id: Uuid) -> ApiResult<()> {
    sqlx::query!("DELETE FROM messages WHERE id = $1;", message_id)
        .execute(pool)
        .await?;
    Ok(())
}
