use crate::state::AppState;
use axum::{
    Router,
    extract::Query,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;

mod auth;
mod channels;
mod hosts;
mod memberships;
mod messages;
mod servers;
mod users;

/// Creates a router for all API endpoints.
pub fn router() -> Router<AppState> {
    Router::new()
        // Mount auth router (includes OIDC discovery and auth endpoints)
        .merge(auth::router())
        // Mount federation router (server-to-server endpoints)
        .nest("/federation", federation_router())
        // API routes
        .route("/ping", get(ping))
        .route("/users", get(users::get_all).post(users::create))
        .route("/users/find", get(users::get_by_name_and_domain))
        .route("/users/{user_id}", get(users::get_by_id))
        .route(
            "/users/{user_id}/domains",
            get(users::get_user_associated_domains),
        )
        .route("/users/{user_id}/servers", get(memberships::get_by_user))
        .route("/messages", get(messages::get_all))
        .route(
            "/servers/{server_id}/channels/{channel_id}/messages/{message_id}",
            get(messages::get_by_id),
        )
        .route("/channels", get(channels::get_all))
        .route(
            "/servers/{server_id}/channels/{channel_id}",
            get(channels::get_by_id),
        )
        .route(
            "/servers/{server_id}/channels/{channel_id}/messages",
            get(messages::get_by_channel).post(messages::create),
        )
        .route("/servers", get(servers::get_all).post(servers::create))
        .route("/servers/{server_id}", get(servers::get_by_id))
        .route(
            "/servers/{server_id}/channels",
            get(channels::get_by_server).post(channels::create),
        )
        .route(
            "/servers/{server_id}/messages",
            get(messages::get_by_server),
        )
        .route(
            "/servers/{server_id}/with_channels",
            get(servers::get_with_channels),
        )
        .route(
            "/servers/{server_id}/users",
            get(memberships::get_members_by_server).post(memberships::create),
        )
        .route(
            "/servers/{server_id}/users/{user_id}",
            get(memberships::get_by_user_and_server),
        )
        .route("/hosts", get(hosts::get_all))
        .route("/hosts/{domain}", get(hosts::get_by_domain))
}

/// Creates a router for all federation endpoints (server-to-server).
pub fn federation_router() -> Router<AppState> {
    Router::new()
        .route(
            "/servers/{server_id}/users",
            post(memberships::federated::create),
        )
        .route("/servers", post(servers::federated::create))
        .route(
            "/servers/{server_id}/with_channels",
            get(servers::federated::get_with_channels),
        )
        .route(
            "/servers/{server_id}/channels",
            post(channels::federated::create)
                .get(channels::federated::get_by_server),
        )
        .route("/channels", get(channels::federated::get_all))
        .route(
            "/servers/{server_id}/channels/{channel_id}",
            get(channels::federated::get_by_id),
        )
        .route("/messages", get(messages::federated::get_all))
        .route(
            "/servers/{server_id}/messages",
            get(messages::federated::get_by_server),
        )
        .route(
            "/servers/{server_id}/channels/{channel_id}/messages",
            post(messages::federated::create)
                .get(messages::federated::get_by_channel),
        )
        .route(
            "/servers/{server_id}/channels/{channel_id}/messages/{message_id}",
            get(messages::federated::get_by_id),
        )
}

#[derive(Deserialize, Debug)]
pub struct PingParams {
    id: Option<i32>,
    msg: Option<String>,
}

pub async fn ping(Query(params): Query<PingParams>) -> impl IntoResponse {
    let msg_part = match params.msg {
        Some(msg) => format!(": \"{msg}\""),
        None => "".to_string(),
    };
    let id_part = match params.id {
        Some(id) => format!(" ({id})"),
        None => "".to_string(),
    };
    let message = format!("pong{id_part}{msg_part}");
    println!("{message}");
    message
}
