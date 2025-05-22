use crate::{
    db::DbPool,
    error::ApiError,
    models::{NewServerMember, ServerMember, ServerRole, User},
};
use sqlx::{types::Json};
use time::OffsetDateTime;
use uuid::Uuid;

/// An intermediate type needed because of sqlx limitations
#[derive(Debug)]
struct ServerMemberRow {
    pub user: Option<Json<User>>,
    pub role: ServerRole,
    pub created_at: OffsetDateTime,
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
    new_member: &NewServerMember,
) -> Result <ServerMember, ApiError> {
    sqlx::query!(
        r#"
        INSERT INTO server_users (server_id, user_id, role)
        VALUES ($1, $2, $3);
        "#,
        server_id,
        new_member.user_id,
        new_member.role as ServerRole,
    )
    .execute(pool)
    .await?;
    get_server_member(pool, server_id, new_member.user_id).await
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
