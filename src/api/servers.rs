use crate::{
    db::DbPool,
    db_queries::{get_all_servers, get_server_with_channels},
    error::ApiError,
};
use axum::{extract::{Path, State}, response::IntoResponse, Json};
use uuid::Uuid;
use std::sync::Arc;

pub async fn list_servers(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    get_all_servers(&pool).await.map(Json)
}

pub async fn get_server_by_id_handler(
    State(pool): State<Arc<DbPool>>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    get_server_with_channels(&pool, server_id).await.map(Json)
}
