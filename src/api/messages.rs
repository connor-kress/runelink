use crate::{db::DbPool, db_queries::get_all_messages, utils::map_diesel_error};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn list_messages(
    State(pool): State<Arc<DbPool>>,
) -> impl IntoResponse {
    let pool = pool.clone();
    let messages_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .expect("couldn't get db connection from pool");
        return get_all_messages(&mut conn);
    })
    .await
    .unwrap();

    return match messages_result {
        Ok(messages) => Json(messages).into_response(),
        Err(e) => map_diesel_error(e).into_response(),
    };
}
