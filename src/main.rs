use axum::{routing::get, Router};
use tokio::net::TcpListener;
mod api;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to create TCP listener");

    let app = Router::new().route("/api/ping", get(api::ping));

    println!("Starting server");
    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}
