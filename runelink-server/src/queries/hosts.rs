use runelink_types::{Host, UserRef};

use crate::{db::DbPool, error::ApiResult};

pub async fn get_all(pool: &DbPool) -> ApiResult<Vec<Host>> {
    let hosts =
        sqlx::query_as!(Host, "SELECT * FROM hosts ORDER BY user_count DESC;",)
            .fetch_all(pool)
            .await?;
    Ok(hosts)
}

pub async fn get_by_domain(pool: &DbPool, domain: &str) -> ApiResult<Host> {
    let host = sqlx::query_as!(
        Host,
        "SELECT * FROM hosts WHERE domain = $1;",
        domain,
    )
    .fetch_one(pool)
    .await?;
    Ok(host)
}

pub async fn get_user_associated_domains(
    pool: &DbPool,
    user_ref: &UserRef,
) -> ApiResult<Vec<String>> {
    let domains = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT s.domain
        FROM user_remote_server_memberships m
        JOIN cached_remote_servers s ON s.id = m.remote_server_id
        WHERE m.user_name = $1 AND m.user_domain = $2
        ORDER BY s.domain ASC;
        "#,
        user_ref.name,
        user_ref.domain,
    )
    .fetch_all(pool)
    .await?;
    Ok(domains)
}
