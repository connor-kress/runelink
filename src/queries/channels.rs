use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::Channel;
use uuid::Uuid;

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
