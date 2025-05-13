use crate::{
    db::{get_conn, DbPool},
    db_queries::{get_users, insert_user},
    error::ApiError,
    models::NewUser,
};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn list_users(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = get_conn(&pool)?;
        get_users(&mut conn)
    })
    .await
    .map_err(ApiError::from)?
    .map(Json)
}

pub async fn create_user(
    State(pool): State<Arc<DbPool>>,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = get_conn(&pool)?;
        insert_user(&mut conn, &new_user)
    })
    .await
    .map_err(ApiError::from)?
    .map(Json)
}
