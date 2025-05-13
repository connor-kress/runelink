use crate::{
    db::DbPool,
    db_queries::{get_users, insert_user},
    models::NewUser,
};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn list_users(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let pool = pool.clone();
    let users_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        return get_users(&mut conn);
    })
    .await
    .unwrap();

    return match users_result {
        Ok(users) => Json(users).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {:?}", e),
        )
            .into_response(),
    };
}

pub async fn create_user(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let new_user = NewUser {
        name: "connor".into(),
        domain: "textbookheaven.org".into(),
    };
    let pool = pool.clone();
    let user_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        return insert_user(&mut conn, &new_user);
    })
    .await
    .unwrap();

    return match user_result {
        Ok(users) => Json(users).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {:?}", e),
        )
            .into_response(),
    };
}
