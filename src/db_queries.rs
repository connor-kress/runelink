use crate::db::DbPool;
use crate::error::ApiError;
use crate::models::{FlatMessage, Message, NewUser, User};
use uuid::Uuid;

pub async fn insert_user(
    pool: &DbPool,
    new_user: &NewUser,
) -> Result<User, ApiError> {
    sqlx::query_as::<_, User>(r#"
        INSERT INTO users (name, domain)
        VALUES ($1, $2)
        RETURNING name, domain, created_at;
    "#)
    .bind(&new_user.name)
    .bind(&new_user.domain)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_all_users(pool: &DbPool) -> Result<Vec<User>, ApiError> {
    sqlx::query_as::<_, User>("SELECT name, domain, created_at FROM users;")
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)
}

pub async fn get_all_messages(
    pool: &DbPool,
) -> Result<Vec<Message>, ApiError> {
    let flats: Vec<FlatMessage> = sqlx::query_as::<_, FlatMessage>(r#"
        SELECT
            m.*,
            su.created_at AS sender_created_at,
            ru.created_at AS recipient_created_at
        FROM messages m
        LEFT JOIN users su ON su.name = m.sender_name
                          AND su.domain = m.sender_domain
        LEFT JOIN users ru ON ru.name = m.recipient_name
                          AND ru.domain = m.recipient_domain
        ORDER BY m.created_at DESC;
    "#)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(flats.into_iter().map(Message::from).collect())
}

pub async fn get_message_by_id(
    pool: &DbPool,
    msg_id: Uuid,
) -> Result<Message, ApiError> {
    sqlx::query_as::<_, FlatMessage>(r#"
        SELECT
            m.*,
            su.created_at AS sender_created_at,
            ru.created_at AS recipient_created_at
        FROM messages m
        LEFT JOIN users su ON su.name = m.sender_name
                          AND su.domain = m.sender_domain
        LEFT JOIN users ru ON ru.name = m.recipient_name
                          AND ru.domain = m.recipient_domain
        WHERE m.id = $1;
    "#)
    .bind(msg_id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
    .map(Message::from)
}
