use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::{Channel, Server, ServerWithChannels};
use uuid::Uuid;

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
