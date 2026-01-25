use runelink_types::LocalAccount;
use uuid::Uuid;

use crate::{db::DbPool, error::ApiResult};

pub async fn insert(
    pool: &DbPool,
    user_id: Uuid,
    password_hash: &str,
) -> ApiResult<LocalAccount> {
    let local_account = sqlx::query_as!(
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
    .await?;
    Ok(local_account)
}

pub async fn get_by_user(
    pool: &DbPool,
    user_id: Uuid,
) -> ApiResult<LocalAccount> {
    let local_account = sqlx::query_as!(
        LocalAccount,
        "SELECT * FROM local_accounts WHERE user_id = $1;",
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(local_account)
}

#[allow(dead_code)]
pub async fn delete_by_user(pool: &DbPool, user_id: Uuid) -> ApiResult<()> {
    sqlx::query!("DELETE FROM local_accounts WHERE user_id = $1;", user_id)
        .execute(pool)
        .await?;
    Ok(())
}
