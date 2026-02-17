use runelink_types::{Host, UserRef};

use crate::{db::DbPool, error::ApiResult};

pub async fn get_user_associated_domains(
    pool: &DbPool,
    user_ref: &UserRef,
) -> ApiResult<Vec<String>> {
    let domains = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT s.host
        FROM user_remote_server_memberships m
        JOIN cached_remote_servers s ON s.id = m.remote_server_id
        WHERE m.user_name = $1 AND m.user_host = $2
        ORDER BY s.host ASC;
        "#,
        user_ref.name,
        user_ref.domain,
    )
    .fetch_all(pool)
    .await?;
    Ok(domains)
}
