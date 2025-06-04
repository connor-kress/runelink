use crate::{error::ApiError, state::AppState};
use runelink_types::{NewServer, Server};
use time::OffsetDateTime;
use uuid::Uuid;

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
    fn into_server(self, domain: &str) -> Server {
        Server {
            id: self.id,
            domain: domain.to_string(),
            title: self.title,
            description: self.description,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

pub async fn insert_server(
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
    Ok(row.into_server(&state.config.local_domain))
}

pub async fn get_server_by_id(
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
    Ok(row.into_server(&state.config.local_domain))
}

pub async fn get_all_servers(
    state: &AppState,
) -> Result<Vec<Server>, ApiError> {
    let rows = sqlx::query_as!(
        LocalServerRow,
        "SELECT * FROM servers",
    )
    .fetch_all(state.db_pool.as_ref())
    .await?;
    let servers = rows
        .into_iter()
        .map(|row| row.into_server(&state.config.local_domain))
        .collect();
    Ok(servers)
}
