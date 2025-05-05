use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
mod api;
mod db;
mod models;
mod schema;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let pool = Arc::new(db::get_pool());

    let app = Router::new()
        .route("/api/ping", get(api::ping))
        .route("/api/users", get(api::list_users))
        .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to create TCP listener");

    println!("Starting server");
    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}
