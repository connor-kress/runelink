use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

mod channels;
mod messages;
mod servers;
mod users;
pub use channels::*;
pub use messages::*;
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
    return message;
}
