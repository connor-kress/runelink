use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
mod api;
mod db;
mod db_queries;
mod error;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let pool = Arc::new(db::get_pool().await?);

    let app = Router::new()
        .route("/api/ping", get(api::ping))
        .route("/api/users", get(api::list_users).post(api::create_user))
        .route("/api/messages", get(api::list_messages))
        .with_state(pool.clone());

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    println!("Starting server on 0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
