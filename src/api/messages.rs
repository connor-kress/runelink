use crate::{
    db::DbPool,
    db_queries::get_all_messages,
    error::ApiError,
};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn list_messages(
    State(pool): State<Arc<DbPool>>,
) -> Result<impl IntoResponse, ApiError> {
    get_all_messages(&pool).await.map(Json)
}
