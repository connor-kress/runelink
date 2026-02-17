#![allow(dead_code)]

use runelink_types::{
    NewServerMembership, Server, ServerMember, ServerMembership, ServerRole,
    User, UserRef,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    config::ServerConfig,
    db::DbPool,
    error::{ApiError, ApiResult},
    state::AppState,
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
    user_name: Option<String>,
    user_domain: Option<String>,
    role: Option<ServerRole>,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
    synced_at: Option<OffsetDateTime>,
}

impl ServerMembershipRow {
    fn try_into_server_membership(
        self,
        config: &ServerConfig,
    ) -> ApiResult<ServerMembership> {
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
            user_ref: UserRef::new(
                self.user_name.ok_or_else(get_error)?,
                self.user_domain.ok_or_else(get_error)?,
            ),
            role: self.role.ok_or_else(get_error)?,
            joined_at: self.created_at.ok_or_else(get_error)?,
            updated_at: self.updated_at.ok_or_else(get_error)?,
            synced_at: self.synced_at,
        })
    }
}

pub async fn insert_local(
    pool: &DbPool,
    new_membership: &NewServerMembership,
) -> ApiResult<ServerMember> {
    sqlx::query!(
        r#"
        INSERT INTO server_users (server_id, user_name, user_domain, role)
        VALUES ($1, $2, $3, $4);
        "#,
        new_membership.server_id,
        new_membership.user_ref.name,
        new_membership.user_ref.domain,
        new_membership.role as ServerRole,
    )
    .execute(pool)
    .await?;
    get_local_member_by_user_and_server(
        pool,
        new_membership.server_id,
        new_membership.user_ref.clone(),
    )
    .await
}

pub async fn insert_remote(
    pool: &DbPool,
    membership: &ServerMembership,
) -> ApiResult<ServerMembership> {
    sqlx::query!(
        r#"
        INSERT INTO user_remote_server_memberships (
            user_name, user_domain, remote_server_id, role, remote_created_at,
            remote_updated_at, synced_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#,
        membership.user_ref.name,
        membership.user_ref.domain,
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
        WHERE m.user_name = $1 AND m.user_domain = $2 AND m.remote_server_id = $3
        "#,
        membership.user_ref.name,
        membership.user_ref.domain,
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
        user_ref: membership.user_ref.clone(),
        role: row.role,
        joined_at: row.membership_created_at,
        updated_at: row.membership_updated_at,
        synced_at: Some(row.synced_at),
    })
}

pub async fn get_local_member_by_user_and_server(
    pool: &DbPool,
    server_id: Uuid,
    user_ref: UserRef,
) -> ApiResult<ServerMember> {
    sqlx::query_as!(
        ServerMemberRow,
        r#"
        SELECT
            to_jsonb(u) "user: Json<User>",
            su.role AS "role: ServerRole",
            su.created_at,
            su.updated_at
        FROM users u
        JOIN server_users su ON u.name = su.user_name AND u.domain = su.user_domain
        WHERE su.server_id = $1 AND su.user_name = $2 AND su.user_domain = $3
        ORDER BY u.name, u.domain
        "#,
        server_id,
        user_ref.name,
        user_ref.domain,
    )
    .fetch_one(pool)
    .await?
    .try_into()
}

pub async fn get_remote_member_by_user_and_server(
    pool: &DbPool,
    server_id: Uuid,
    user_ref: UserRef,
) -> ApiResult<ServerMember> {
    sqlx::query_as!(
        ServerMemberRow,
        r#"
        SELECT
            to_jsonb(u) "user: Json<User>",
            m.role AS "role: ServerRole",
            m.remote_created_at AS created_at,
            m.remote_updated_at AS updated_at
        FROM users u
        JOIN user_remote_server_memberships m
          ON u.name = m.user_name AND u.domain = m.user_domain
        WHERE m.remote_server_id = $1 AND m.user_name = $2 AND m.user_domain = $3
        "#,
        server_id,
        user_ref.name,
        user_ref.domain,
    )
    .fetch_one(pool)
    .await?
    .try_into()
}

pub async fn get_member_by_user_and_server(
    pool: &DbPool,
    server_id: Uuid,
    user_ref: UserRef,
) -> ApiResult<ServerMember> {
    match get_local_member_by_user_and_server(pool, server_id, user_ref.clone())
        .await
    {
        Ok(member) => Ok(member),
        Err(_) => {
            // Fall back to remote membership cache
            get_remote_member_by_user_and_server(pool, server_id, user_ref)
                .await
        }
    }
}

pub async fn get_members_by_server(
    pool: &DbPool,
    server_id: Uuid,
) -> ApiResult<Vec<ServerMember>> {
    sqlx::query_as!(
        ServerMemberRow,
        r#"
        SELECT
            to_jsonb(u) "user: Json<User>",
            su.role AS "role: ServerRole",
            su.created_at,
            su.updated_at
        FROM users u
        JOIN server_users su ON u.name = su.user_name AND u.domain = su.user_domain
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
    user: UserRef,
) -> ApiResult<ServerMembership> {
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
            AND su.user_name = $2 AND su.user_domain = $3
        "#,
        server_id,
        user.name,
        user.domain,
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
        user_ref: user,
        role: row.role,
        joined_at: row.membership_created_at,
        updated_at: row.membership_updated_at,
        synced_at: None,
    })
}

pub async fn get_by_user(
    state: &AppState,
    user: UserRef,
) -> ApiResult<Vec<ServerMembership>> {
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
            su.user_name AS user_name,
            su.user_domain AS user_domain,
            su.role AS "role!: Option<ServerRole>",
            su.created_at,
            su.updated_at,
            NULL::TIMESTAMPTZ AS synced_at
        FROM servers s
        JOIN server_users su ON s.id = su.server_id
        WHERE su.user_name = $1 AND su.user_domain = $2

        UNION ALL

        -- Cached remote server memberships
        SELECT
            crs.id AS server_id,
            crs.title AS server_title,
            crs.description AS server_description,
            crs.domain AS server_domain_from_db,
            crs.remote_created_at AS server_created_at,
            crs.remote_updated_at AS server_updated_at,
            ursm.user_name AS user_name,
            ursm.user_domain AS user_domain,
            ursm.role AS "role!: Option<ServerRole>",
            ursm.remote_created_at AS created_at,
            ursm.remote_updated_at AS updated_at,
            ursm.synced_at AS synced_at
        FROM cached_remote_servers crs
        JOIN user_remote_server_memberships ursm
            ON crs.id = ursm.remote_server_id
        WHERE ursm.user_name = $1 AND ursm.user_domain = $2

        ORDER BY server_title ASC
        "#,
        user.name,
        user.domain,
    )
    .fetch_all(state.db_pool.as_ref())
    .await?;

    rows.into_iter()
        .map(|row| row.try_into_server_membership(&state.config))
        .collect()
}

/// Get distinct remote server domains where a user has memberships.
pub async fn get_remote_server_domains_for_user(
    pool: &DbPool,
    user: UserRef,
) -> ApiResult<Vec<String>> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT crs.domain
        FROM cached_remote_servers crs
        JOIN user_remote_server_memberships ursm
            ON crs.id = ursm.remote_server_id
        WHERE ursm.user_name = $1 AND ursm.user_domain = $2
        "#,
        user.name,
        user.domain,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| row.domain).collect())
}

/// Delete a local server membership.
pub async fn delete_local(
    pool: &DbPool,
    server_id: Uuid,
    user: UserRef,
) -> ApiResult<()> {
    let result = sqlx::query!(
        r#"
        DELETE FROM server_users
        WHERE server_id = $1 AND user_name = $2 AND user_domain = $3
        "#,
        server_id,
        user.name,
        user.domain,
    )
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

pub async fn delete_remote(
    pool: &DbPool,
    server_id: Uuid,
    user: UserRef,
) -> ApiResult<()> {
    let result = sqlx::query!(
        r#"
        DELETE FROM user_remote_server_memberships
        WHERE remote_server_id = $1 AND user_name = $2 AND user_domain = $3
        "#,
        server_id,
        user.name,
        user.domain,
    )
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}
