use axum::{Router, routing::get};
use config::ServerConfig;
use sqlx::migrate::Migrator;
use state::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::key_manager::KeyManager;

mod api;
mod auth;
mod bearer_auth;
mod config;
mod crypto;
mod db;
mod error;
mod jwks_resolver;
mod key_manager;
mod ops;
mod queries;
mod state;

// Embed all sql migrations in binary
static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = Arc::new(ServerConfig::from_env()?);
    let pool = Arc::new(db::get_pool(config.as_ref()).await?);
    let http_client = reqwest::Client::new();
    let key_manager = KeyManager::load_or_generate(config.key_dir.clone())?;

    let app_state = AppState {
        config: config.clone(),
        db_pool: pool.clone(),
        http_client,
        key_manager,
        jwks_cache: Arc::new(tokio::sync::RwLock::new(
            std::collections::HashMap::new(),
        )),
    };

    MIGRATOR.run(pool.as_ref()).await?;
    println!("Migrations are up to date.");

    let app = Router::new()
        // Mount auth router (includes OIDC discovery and auth endpoints)
        .merge(api::auth::router())
        // Mount federation router (server-to-server endpoints)
        .nest("/federation", api::federation_router())
        // API routes
        .route("/ping", get(api::ping))
        .route("/users", get(api::list_users).post(api::create_user))
        .route("/users/find", get(api::find_user_by_name_domain_handler))
        .route("/users/{user_id}", get(api::get_user_by_id_handler))
        .route(
            "/users/{user_id}/domains",
            get(api::get_user_associated_domains),
        )
        .route(
            "/users/{user_id}/servers",
            get(api::list_server_memberships_by_user),
        )
        .route("/messages", get(api::list_messages))
        .route(
            "/servers/{server_id}/channels/{channel_id}/messages/{message_id}",
            get(api::get_message_by_id_handler),
        )
        .route("/channels", get(api::list_channels))
        .route(
            "/servers/{server_id}/channels/{channel_id}",
            get(api::get_channel_by_id_handler),
        )
        .route(
            "/servers/{server_id}/channels/{channel_id}/messages",
            get(api::list_messages_by_channel).post(api::create_message),
        )
        .route("/servers", get(api::list_servers).post(api::create_server))
        .route("/servers/{server_id}", get(api::get_server_by_id_handler))
        .route(
            "/servers/{server_id}/channels",
            get(api::list_channels_by_server).post(api::create_channel),
        )
        .route(
            "/servers/{server_id}/messages",
            get(api::list_messages_by_server),
        )
        .route(
            "/servers/{server_id}/with_channels",
            get(api::get_server_with_channels_handler),
        )
        .route(
            "/servers/{server_id}/users",
            get(api::list_server_members).post(api::add_server_member),
        )
        .route(
            "/servers/{server_id}/users/{user_id}",
            get(api::get_server_member),
        )
        .route("/hosts", get(api::list_hosts))
        .route("/hosts/{domain}", get(api::get_host))
        .with_state(app_state);

    let ip_addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&ip_addr).await?;

    println!("Starting server on {}", ip_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
