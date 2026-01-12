use crate::{db::DbPool, error::ApiError, state::AppState};
use runelink_types::{
    NewServerMembership, Server, ServerMember, ServerMembership, ServerRole,
    User,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use time::OffsetDateTime;
use uuid::Uuid;

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

pub async fn add_user_to_server(
    pool: &DbPool,
    server_id: Uuid,
    new_membership: &NewServerMembership,
) -> Result<ServerMember, ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO server_users (server_id, user_id, role)
        VALUES ($1, $2, $3);
        "#,
        server_id,
        new_membership.user_id,
        new_membership.role as ServerRole,
    )
    .execute(pool)
    .await?;
    get_server_member(pool, server_id, new_membership.user_id).await
}

pub async fn get_server_member(
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

pub async fn get_all_server_members(
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

pub async fn upsert_cached_remote_server(
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

pub async fn insert_user_remote_server_membership(
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

pub async fn get_local_server_membership(
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
