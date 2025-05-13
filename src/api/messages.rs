use crate::{
    db::{get_conn, DbPool},
    db_queries::get_all_messages,
    error::ApiError,
};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn list_messages(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = get_conn(&pool)?;
        get_all_messages(&mut conn)
    })
    .await
    .map_err(ApiError::from)?
    .map(Json)
}
