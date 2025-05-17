use crate::{
    db::DbPool,
    error::ApiError,
    models::{NewServer, Server},
};
use uuid::Uuid;

pub async fn insert_server(
    pool: &DbPool,
    new_server: &NewServer,
) -> Result<Server, ApiError> {
    let new_id = Uuid::new_v4();
    sqlx::query_as!(
        Server,
        r#"
        INSERT INTO servers (id, title, description)
        VALUES ($1, $2, $3)
        RETURNING *;
        "#,
        new_id,
        new_server.title,
        new_server.description,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_server_by_id(
    pool: &DbPool,
    server_id: Uuid,
) -> Result<Server, ApiError> {
    sqlx::query_as!(
        Server,
        "SELECT * FROM servers WHERE id = $1;",
        server_id,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

pub async fn get_all_servers(
    pool: &DbPool,
) -> Result<Vec<Server>, ApiError> {
    sqlx::query_as!(
        Server,
        "SELECT * FROM servers",
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)
}
