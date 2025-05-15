use axum::{routing::get, Router};
use sqlx::migrate::Migrator;
use std::sync::Arc;
use tokio::net::TcpListener;
mod api;
mod db;
mod db_queries;
mod error;
mod models;

// Embed all sql migrations in binary
static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let pool = Arc::new(db::get_pool().await?);

    MIGRATOR.run(pool.as_ref()).await?;
    println!("Migrations are up to date.");

    let app = Router::new()
        .route("/api/ping", get(api::ping))
        .route("/api/users", get(api::list_users).post(api::create_user))
        .route("/api/messages", get(api::list_messages))
        .route(
            "/api/messages/id/{message_id}",
            get(api::get_message_by_id_handler),
        )
        .route(
            "/api/channels/id/{message_id}",
            get(api::get_channel_by_id_handler),
        )
        .route("/api/servers", get(api::list_servers))
        .route( "/api/servers/id/{server_id}",
            get(api::get_server_by_id_handler),
        )
        .with_state(pool.clone());

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    println!("Starting server on 0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
