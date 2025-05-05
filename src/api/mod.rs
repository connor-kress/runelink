use crate::{db::DbPool, models::User, schema::users};
use axum::{extract::Query, extract::State, response::IntoResponse, Json};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PingParams {
    id: Option<i32>,
    msg: Option<String>,
}

pub async fn ping(Query(params): Query<PingParams>) -> impl IntoResponse {
    let user_msg = match params.msg {
        Some(msg) => format!("\"{}\"", msg),
        None => "No message".to_owned(),
    };
    let message = match params.id {
        Some(id) => format!("pong ({}): {}", id, user_msg),
        None => format!("pong: {}", user_msg),
    };
    println!("{}", message);
    return message;
}

pub async fn list_users(State(pool): State<Arc<DbPool>>) -> impl IntoResponse {
    let pool = pool.clone();
    let users_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        return users::table.load::<User>(&mut conn);
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
