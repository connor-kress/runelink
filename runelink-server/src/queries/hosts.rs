use crate::{db::DbPool, error::ApiError};
use runelink_types::Host;

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
