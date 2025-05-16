use crate::{db::DbPool, error::ApiError, models::Server};
use uuid::Uuid;

pub async fn get_server_by_id(
    pool: &DbPool,
    server_id: Uuid,
) -> Result<Server, ApiError> {
    sqlx::query_as!(
        Server,
        "SELECT * FROM servers WHERE id = $1;",
        server_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
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
