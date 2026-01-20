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

    // Initialize logger - reads RUST_LOG environment variable
    // Examples: RUST_LOG=info, RUST_LOG=debug, RUST_LOG=runelink_server=debug
    // Defaults to info level if RUST_LOG is not set
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

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
    log::info!("Migrations are up to date.");

    let app = api::router().with_state(app_state);

    let ip_addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&ip_addr).await?;

    log::info!("Starting server on {ip_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
