use crate::{config::ServerConfig, error::ApiError, state::AppState};
use runelink_types::{NewServer, Server, ServerMembership, ServerRole};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
struct ServerMembershipRow {
    server_id: Option<Uuid>,
    server_title: Option<String>,
    server_description: Option<String>,
    server_domain_from_db: Option<String>,
    server_created_at: Option<OffsetDateTime>,
    server_updated_at: Option<OffsetDateTime>,
    role: Option<ServerRole>,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
    synced_at: Option<OffsetDateTime>,
}

impl ServerMembershipRow {
    fn try_into_server_membership(
        self,
        user_id: Uuid,
        config: &ServerConfig,
    ) -> Result<ServerMembership, ApiError> {
        let server_domain = self
            .server_domain_from_db
            .unwrap_or_else(|| config.local_domain());

        // Needed because of weird sqlx limitations (or misuse)
        let get_error = || ApiError::Unknown("Sqlx conversion error".into());
        Ok(ServerMembership {
            server: Server {
                id: self.server_id.ok_or_else(get_error)?,
                title: self.server_title.ok_or_else(get_error)?,
                description: self.server_description,
                domain: server_domain,
                created_at: self.server_created_at.ok_or_else(get_error)?,
                updated_at: self.server_updated_at.ok_or_else(get_error)?,
            },
            user_id,
            role: self.role.ok_or_else(get_error)?,
            joined_at: self.created_at.ok_or_else(get_error)?,
            updated_at: self.updated_at.ok_or_else(get_error)?,
            synced_at: self.synced_at,
        })
    }
}

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
    Ok(row.into_server(&state.config))
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
    Ok(row.into_server(&state.config))
}

pub async fn get_all_servers(
    state: &AppState,
) -> Result<Vec<Server>, ApiError> {
    let rows = sqlx::query_as!(LocalServerRow, "SELECT * FROM servers",)
        .fetch_all(state.db_pool.as_ref())
        .await?;
    let servers = rows
        .into_iter()
        .map(|row| row.into_server(&state.config))
        .collect();
    Ok(servers)
}

pub async fn get_all_memberships_for_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<ServerMembership>, ApiError> {
    let rows = sqlx::query_as!(
        ServerMembershipRow,
        r#"
        -- Local server memberships
        SELECT
            s.id AS server_id,
            s.title AS server_title,
            s.description AS server_description,
            NULL::TEXT AS server_domain_from_db,
            s.created_at AS server_created_at,
            s.updated_at AS server_updated_at,
            su.role AS "role!: Option<ServerRole>",
            su.created_at,
            su.updated_at,
            NULL::TIMESTAMPTZ AS synced_at
        FROM servers s
        JOIN server_users su ON s.id = su.server_id
        WHERE su.user_id = $1

        UNION ALL

        -- Cached remote server memberships
        SELECT
            crs.id AS server_id,
            crs.title AS server_title,
            crs.description AS server_description,
            crs.domain AS server_domain_from_db,
            crs.remote_created_at AS server_created_at,
            crs.remote_updated_at AS server_updated_at,
            ursm.role AS "role!: Option<ServerRole>",
            ursm.remote_created_at AS created_at,
            ursm.remote_updated_at AS updated_at,
            ursm.synced_at AS synced_at
        FROM cached_remote_servers crs
        JOIN user_remote_server_memberships ursm
            ON crs.id = ursm.remote_server_id
        WHERE ursm.user_id = $1

        ORDER BY server_title ASC
        "#,
        user_id,
    )
    .fetch_all(state.db_pool.as_ref())
    .await?;

    rows.into_iter()
        .map(|row| row.try_into_server_membership(user_id, &state.config))
        .collect()
}
