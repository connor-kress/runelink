use runelink_types::RefreshToken;

use crate::{db::DbPool, error::ApiError};

pub async fn insert_refresh(
    pool: &DbPool,
    rt: &RefreshToken,
) -> Result<RefreshToken, ApiError> {
    sqlx::query_as!(
        RefreshToken,
        r#"
        INSERT INTO refresh_tokens (token, user_id, client_id, issued_at,
                                    expires_at, revoked)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING token, user_id, client_id, issued_at, expires_at, revoked
        "#,
        rt.token,
        rt.user_id,
        rt.client_id,
        rt.issued_at,
        rt.expires_at,
        rt.revoked,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_refresh(
    pool: &DbPool,
    token_str: &str,
) -> Result<RefreshToken, ApiError> {
    sqlx::query_as!(
        RefreshToken,
        r#"
        SELECT token, user_id, client_id, issued_at, expires_at, revoked
        FROM refresh_tokens
        WHERE token = $1
        "#,
        token_str,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

#[allow(dead_code)]
pub async fn revoke_refresh(
    pool: &DbPool,
    token_str: &str,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked = TRUE
        WHERE token = $1
        "#,
        token_str,
    )
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(ApiError::from)
}
