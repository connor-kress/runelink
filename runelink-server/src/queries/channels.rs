use runelink_types::{Channel, NewChannel};
use uuid::Uuid;

use crate::{db::DbPool, error::ApiError};

pub async fn insert(
    pool: &DbPool,
    server_id: Uuid,
    new_channel: &NewChannel,
) -> Result<Channel, ApiError> {
    sqlx::query_as!(
        Channel,
        r#"
        INSERT INTO channels (server_id, title, description)
        VALUES ($1, $2, $3)
        RETURNING *;
        "#,
        server_id,
        new_channel.title,
        new_channel.description,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_by_id(
    pool: &DbPool,
    channel_id: Uuid,
) -> Result<Channel, ApiError> {
    sqlx::query_as!(
        Channel,
        "SELECT * FROM channels WHERE id = $1;",
        channel_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_all(pool: &DbPool) -> Result<Vec<Channel>, ApiError> {
    sqlx::query_as!(Channel, "SELECT * FROM channels",)
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)
}

pub async fn get_by_server(
    pool: &DbPool,
    server_id: Uuid,
) -> Result<Vec<Channel>, ApiError> {
    sqlx::query_as!(
        Channel,
        r#"
        SELECT * FROM channels
        WHERE server_id = $1
        ORDER BY created_at;
        "#,
        server_id,
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn delete(pool: &DbPool, channel_id: Uuid) -> Result<(), ApiError> {
    sqlx::query!("DELETE FROM channels WHERE id = $1;", channel_id)
        .execute(pool)
        .await
        .map_err(ApiError::from)?;
    Ok(())
}
