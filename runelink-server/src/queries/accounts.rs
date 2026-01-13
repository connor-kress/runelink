use runelink_types::LocalAccount;
use uuid::Uuid;

use crate::{db::DbPool, error::ApiError};

pub async fn insert(
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

pub async fn get_by_user(
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
