use crate::{
    db::DbPool,
    error::ApiError,
    models::{Channel, NewChannel},
};
use uuid::Uuid;

pub async fn insert_channel(
    pool: &DbPool,
    server_id: Uuid,
    new_channel: &NewChannel,
) -> Result<Channel, ApiError> {
    let new_id = Uuid::new_v4();
    sqlx::query_as!(
        Channel,
        r#"
        INSERT INTO channels (id, server_id, title, description)
        VALUES ($1, $2, $3, $4)
        RETURNING *;
        "#,
        new_id,
        server_id,
        new_channel.title,
        new_channel.description,
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
        channel_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_all_channels(
    pool: &DbPool,
) -> Result<Vec<Channel>, ApiError> {
    sqlx::query_as!(
        Channel,
        "SELECT * FROM channels",
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_channels_by_server(
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
