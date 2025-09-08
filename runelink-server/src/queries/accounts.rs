use crate::{db::DbPool, error::ApiError};
use runelink_types::LocalAccount;
use uuid::Uuid;

pub async fn insert_local_account(
    pool: &DbPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<LocalAccount, ApiError> {
    sqlx::query_as!(
        LocalAccount,
        r#"
        INSERT INTO local_accounts (user_id, password_hash)
        VALUES ($1, $2)
        RETURNING *;
        "#,
        user_id,
        password_hash,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

#[allow(dead_code)]
pub async fn get_local_account(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<LocalAccount, ApiError> {
    sqlx::query_as!(
        LocalAccount,
        "SELECT * FROM local_accounts WHERE user_id = $1;",
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}
