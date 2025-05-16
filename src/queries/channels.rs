use crate::{db::DbPool, error::ApiError, models::Channel};
use uuid::Uuid;

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
