use crate::{db::DbPool, error::ApiError, models::ServerWithChannels, queries};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// GET /api/servers
pub async fn list_servers(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_servers(&pool).await.map(Json)
}

/// GET /api/servers/{id}
pub async fn get_server_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_server_by_id(&pool, server_id).await.map(Json)
}

/// GET /api/servers/{id}/with_channels
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
