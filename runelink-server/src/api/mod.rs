use crate::state::AppState;
use axum::{
    Router,
    extract::Query,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;

pub mod auth;
mod channels;
mod hosts;
mod messages;
mod server_members;
mod servers;
mod users;
pub use channels::*;
pub use hosts::*;
pub use messages::*;
pub use server_members::*;
pub use servers::*;
pub use users::*;

#[derive(Deserialize, Debug)]
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
    message
}

/// Creates a router for all federation endpoints (server-to-server).
pub fn federation_router() -> Router<AppState> {
    Router::new()
        .route("/users/{user_id}", get(users::federated::get_user))
        .route(
            "/servers/{server_id}/memberships",
            post(server_members::federated::create_membership),
        )
}
