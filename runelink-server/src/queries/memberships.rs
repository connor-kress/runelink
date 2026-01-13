use runelink_types::{
    NewServerMembership, Server, ServerMember, ServerMembership, ServerRole,
    User,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    config::ServerConfig, db::DbPool, error::ApiError, state::AppState,
};

/// An intermediate type needed because of sqlx limitations
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
struct ServerMemberRow {
    pub user: Option<Json<User>>,
    pub role: ServerRole,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl TryFrom<ServerMemberRow> for ServerMember {
    type Error = ApiError;

    fn try_from(row: ServerMemberRow) -> Result<Self, Self::Error> {
        let user = row.user.ok_or(ApiError::Unknown("User is null".into()))?.0;
        Ok(ServerMember {
            user,
            role: row.role,
            joined_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

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

pub async fn insert(
    pool: &DbPool,
    new_membership: &NewServerMembership,
) -> Result<ServerMember, ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO server_users (server_id, user_id, role)
        VALUES ($1, $2, $3);
        "#,
        new_membership.server_id,
        new_membership.user_id,
        new_membership.role as ServerRole,
    )
    .execute(pool)
    .await?;
    get_member_by_user_and_server(
        pool,
        new_membership.server_id,
        new_membership.user_id,
    )
    .await
}

pub async fn insert_remote(
    pool: &DbPool,
    membership: &ServerMembership,
) -> Result<ServerMembership, ApiError> {
    // create (or no-op if already exists)
    sqlx::query!(
        r#"
        INSERT INTO user_remote_server_memberships (
            user_id, remote_server_id, role, remote_created_at,
            remote_updated_at, synced_at
        )
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        membership.user_id,
        membership.server.id,
        membership.role as ServerRole,
        membership.joined_at,
        membership.updated_at,
    )
    .execute(pool)
    .await?;

    let row = sqlx::query!(
        r#"
        SELECT
          s.id,
          s.domain,
          s.title,
          s.description,
          s.remote_created_at AS server_created_at,
          s.remote_updated_at AS server_updated_at,
          m.role AS "role: ServerRole",
          m.remote_created_at AS membership_created_at,
          m.remote_updated_at AS membership_updated_at,
          m.synced_at
        FROM cached_remote_servers s
        JOIN user_remote_server_memberships m
          ON s.id = m.remote_server_id
        WHERE m.user_id = $1 AND m.remote_server_id = $2
        "#,
        membership.user_id,
        membership.server.id,
    )
    .fetch_one(pool)
    .await?;

    Ok(ServerMembership {
        server: Server {
            id: row.id,
            domain: row.domain,
            title: row.title,
            description: row.description,
            created_at: row.server_created_at,
            updated_at: row.server_updated_at,
        },
        user_id: membership.user_id,
        role: row.role,
        joined_at: row.membership_created_at,
        updated_at: row.membership_updated_at,
        synced_at: Some(row.synced_at),
    })
}

pub async fn get_member_by_user_and_server(
    pool: &DbPool,
    server_id: Uuid,
    user_id: Uuid,
) -> Result<ServerMember, ApiError> {
    sqlx::query_as!(
        ServerMemberRow,
        r#"
        SELECT
            to_jsonb(u) "user: Json<User>",
            su.role AS "role: ServerRole",
            su.created_at,
            su.updated_at
        FROM users u
        JOIN server_users su ON u.id = su.user_id
        WHERE su.server_id = $1 AND u.id = $2
        ORDER BY u.name, u.domain
        "#,
        server_id,
        user_id,
    )
    .fetch_one(pool)
    .await?
    .try_into()
}

pub async fn get_members_by_server(
    pool: &DbPool,
    server_id: Uuid,
) -> Result<Vec<ServerMember>, ApiError> {
    sqlx::query_as!(
        ServerMemberRow,
        r#"
        SELECT
            to_jsonb(u) "user: Json<User>",
            su.role AS "role: ServerRole",
            su.created_at,
            su.updated_at
        FROM users u
        JOIN server_users su ON u.id = su.user_id
        WHERE su.server_id = $1
        ORDER BY u.name, u.domain
        "#,
        server_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(ServerMember::try_from)
    .collect()
}

pub async fn get_local_by_user_and_server(
    state: &AppState,
    server_id: Uuid,
    user_id: Uuid,
) -> Result<ServerMembership, ApiError> {
    let row = sqlx::query!(
        r#"
        SELECT
            s.id,
            s.title,
            s.description,
            s.created_at AS server_created_at,
            s.updated_at AS server_updated_at,
            su.role AS "role: ServerRole",
            su.created_at AS membership_created_at,
            su.updated_at AS membership_updated_at
        FROM servers s
        JOIN server_users su
            ON s.id = su.server_id
        WHERE s.id = $1
            AND su.user_id = $2
        "#,
        server_id,
        user_id,
    )
    .fetch_one(state.db_pool.as_ref())
    .await?;

    Ok(ServerMembership {
        server: Server {
            id: row.id,
            domain: state.config.local_domain(),
            title: row.title,
            description: row.description,
            created_at: row.server_created_at,
            updated_at: row.server_updated_at,
        },
        user_id,
        role: row.role,
        joined_at: row.membership_created_at,
        updated_at: row.membership_updated_at,
        synced_at: None,
    })
}

pub async fn get_by_user(
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
