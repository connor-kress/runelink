use axum::{
    routing::{get, post},
    Router,
};
use config::ServerConfig;
use sqlx::migrate::Migrator;
use state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
mod auth;
mod api;
mod config;
mod db;
mod queries;
mod error;
mod state;

// Embed all sql migrations in binary
static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = Arc::new(ServerConfig::from_env()?);
    let pool = Arc::new(db::get_pool(config.as_ref()).await?);
    let http_client = reqwest::Client::new();
    let app_state = AppState {
        config: config.clone(),
        db_pool: pool.clone(),
        http_client,
    };

    MIGRATOR.run(pool.as_ref()).await?;
    println!("Migrations are up to date.");

    let app = Router::new()
        .route("/api/ping", get(api::ping))

        .route("/api/users", get(api::list_users).post(api::create_user))
        .route("/api/users/find", get(api::find_user_by_name_domain_handler))
        .route(
            "/api/users/{user_id}",
            get(api::get_user_by_id_handler),
        )
        .route("/api/users/{user_id}/domains",
            get(api::get_user_associated_domains),
        )
        .route("/api/users/{user_id}/servers",
            get(api::list_server_memberships_by_user),
        )

        .route("/api/messages", get(api::list_messages))
        .route(
            "/api/messages/{message_id}",
            get(api::get_message_by_id_handler),
        )

        .route("/api/channels", get(api::list_channels))
        .route("/api/channels/{channel_id}", get(api::get_channel_by_id_handler))
        .route(
            "/api/channels/{channel_id}/messages",
            get(api::list_messages_by_channel).post(api::create_message),
        )

        .route(
            "/api/servers",
            get(api::list_servers).post(api::create_server),
        )
        .route(
            "/api/servers/{server_id}",
            get(api::get_server_by_id_handler),
        )
        .route(
            "/api/servers/{server_id}/channels",
            get(api::list_channels_by_server).post(api::create_channel),
        )
        .route(
            "/api/servers/{server_id}/messages",
            get(api::list_messages_by_server),
        )
        .route(
            "/api/servers/{server_id}/with_channels",
            get(api::get_server_with_channels_handler),
        )
        .route(
            "/api/servers/{server_id}/users",
            get(api::list_server_members).post(api::add_server_member),
        )
        .route(
            "/api/servers/{server_id}/users/{user_id}",
            get(api::get_server_member),
        )
        .route(
            "/api/servers/{server_id}/remote-memberships",
            post(api::create_remote_membership),
        )

        .route("/api/hosts", get(api::list_hosts))
        .route("/api/hosts/{domain}", get(api::get_host))

        .with_state(app_state);

    let ip_addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&ip_addr).await?;

    println!("Starting server on {}", ip_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
