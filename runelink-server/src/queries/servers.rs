use runelink_types::{NewServer, Server};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    config::ServerConfig, db::DbPool, error::ApiError, state::AppState,
};

#[derive(sqlx::FromRow, Debug)]
struct LocalServerRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    // No 'domain' field
}

impl LocalServerRow {
    fn into_server(self, config: &ServerConfig) -> Server {
        Server {
            id: self.id,
            domain: config.local_domain(),
            title: self.title,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub async fn insert(
    state: &AppState,
    new_server: &NewServer,
) -> Result<Server, ApiError> {
    let row = sqlx::query_as!(
        LocalServerRow,
        r#"
        INSERT INTO servers (title, description)
        VALUES ($1, $2)
        RETURNING *;
        "#,
        new_server.title,
        new_server.description,
    )
    .fetch_one(state.db_pool.as_ref())
    .await?;
    Ok(row.into_server(&state.config))
}

pub async fn upsert_remote(
    pool: &DbPool,
    server: &Server,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO cached_remote_servers (
            id, domain, title, description, remote_created_at,
            remote_updated_at, synced_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        ON CONFLICT(id) DO UPDATE
            SET domain = EXCLUDED.domain,
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                remote_created_at = EXCLUDED.remote_created_at,
                remote_updated_at = EXCLUDED.remote_updated_at,
                synced_at = NOW()
        "#,
        server.id,
        server.domain,
        server.title,
        server.description,
        server.created_at,
        server.updated_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn get_by_id(
    state: &AppState,
    server_id: Uuid,
) -> Result<Server, ApiError> {
    let row = sqlx::query_as!(
        LocalServerRow,
        "SELECT * FROM servers WHERE id = $1;",
        server_id,
    )
    .fetch_one(state.db_pool.as_ref())
    .await?;
    Ok(row.into_server(&state.config))
}

pub async fn get_all(state: &AppState) -> Result<Vec<Server>, ApiError> {
    let rows = sqlx::query_as!(LocalServerRow, "SELECT * FROM servers",)
        .fetch_all(state.db_pool.as_ref())
        .await?;
    let servers = rows
        .into_iter()
        .map(|row| row.into_server(&state.config))
        .collect();
    Ok(servers)
}
