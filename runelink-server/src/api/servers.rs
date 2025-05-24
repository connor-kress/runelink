use crate::{auth::AuthBuilder, db::DbPool, error::ApiError, queries};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::{NewServer, ServerWithChannels};
use std::sync::Arc;
use uuid::Uuid;

/// POST /api/servers
pub async fn create_server(
    State(pool): State<Arc<DbPool>>,
    Json(new_server): Json<NewServer>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: get user id/session tokens
    AuthBuilder::new(new_server.user_id)
        .admin()
        .build(&pool)
        .await?;
    queries::insert_server(&pool, &new_server)
        .await
        .map(|server| (StatusCode::CREATED, Json(server)))
}

/// GET /api/servers
pub async fn list_servers(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_servers(&pool).await.map(Json)
}

/// GET /api/servers/{server_id}
pub async fn get_server_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_server_by_id(&pool, server_id).await.map(Json)
}

/// GET /api/servers/{server_id}/with_channels
pub async fn get_server_with_channels_handler(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let (server, channels) = tokio::join!(
        queries::get_server_by_id(&pool, server_id),
        queries::get_channels_by_server(&pool, server_id),
    );
    Ok(Json(ServerWithChannels {
        server: server?,
        channels: channels?,
    }))
}
