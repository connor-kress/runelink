use runelink_types::Host;
use uuid::Uuid;

use crate::{db::DbPool, error::ApiError};

pub async fn get_all(pool: &DbPool) -> Result<Vec<Host>, ApiError> {
    sqlx::query_as!(Host, "SELECT * FROM hosts ORDER BY user_count DESC;",)
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)
}

pub async fn get_by_domain(
    pool: &DbPool,
    domain: &str,
) -> Result<Host, ApiError> {
    sqlx::query_as!(Host, "SELECT * FROM hosts WHERE domain = $1;", domain,)
        .fetch_one(pool)
        .await
        .map_err(ApiError::from)
}

pub async fn get_user_associated_domains(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<String>, ApiError> {
    sqlx::query_scalar!(
        r#"
        SELECT DISTINCT s.domain
        FROM user_remote_server_memberships m
        JOIN cached_remote_servers s ON s.id = m.remote_server_id
        WHERE m.user_id = $1
        ORDER BY s.domain ASC;
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}
