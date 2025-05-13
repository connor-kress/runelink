use crate::{
    db::DbPool,
    db_queries::{get_users, insert_user},
    models::NewUser,
    utils::map_diesel_error,
};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn list_users(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let pool = pool.clone();
    let users_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .expect("couldn't get db connection from pool");
        return get_users(&mut conn);
    })
    .await
    .unwrap();

    return match users_result {
        Ok(users) => Json(users).into_response(),
        Err(e) => map_diesel_error(e).into_response(),
    };
}

pub async fn create_user(
    State(pool): State<Arc<DbPool>>,
    Json(new_user): Json<NewUser>,
) -> impl IntoResponse {
    let pool = pool.clone();
    let user_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .expect("couldn't get db connection from pool");
        return insert_user(&mut conn, &new_user);
    })
    .await
    .unwrap();

    return match user_result {
        Ok(users) => Json(users).into_response(),
        Err(e) => map_diesel_error(e).into_response(),
    };
}
