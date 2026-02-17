use runelink_types::RefreshToken;

use crate::{db::DbPool, error::ApiResult};

pub async fn insert_refresh(
    pool: &DbPool,
    rt: &RefreshToken,
) -> ApiResult<RefreshToken> {
    let refresh_token = sqlx::query_as!(
        RefreshToken,
        r#"
        INSERT INTO refresh_tokens (token, user_name, user_domain, client_id, issued_at,
                                    expires_at, revoked)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING token, user_name, user_domain, client_id, issued_at, expires_at, revoked
        "#,
        rt.token,
        rt.user_name,
        rt.user_domain,
        rt.client_id,
        rt.issued_at,
        rt.expires_at,
        rt.revoked,
    )
    .fetch_one(pool)
    .await?;
    Ok(refresh_token)
}

pub async fn get_refresh(
    pool: &DbPool,
    token_str: &str,
) -> ApiResult<RefreshToken> {
    let refresh_token = sqlx::query_as!(
        RefreshToken,
        r#"
        SELECT token, user_name, user_domain, client_id, issued_at, expires_at, revoked
        FROM refresh_tokens
        WHERE token = $1
        "#,
        token_str,
    )
    .fetch_one(pool)
    .await?;
    Ok(refresh_token)
}

#[allow(dead_code)]
pub async fn revoke_refresh(pool: &DbPool, token_str: &str) -> ApiResult<()> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked = TRUE
        WHERE token = $1
        "#,
        token_str,
    )
    .execute(pool)
    .await?;
    Ok(())
}
