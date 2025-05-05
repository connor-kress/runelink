use axum::response::IntoResponse;
// use serde::Deserialize;

// #[derive(Deserialize)]
// pub struct LoginRequest {
//     pub username: String,
//     pub password: String,
// }

// pub async fn login(Json(_payload): Json<LoginRequest>) -> impl IntoResponse {
//     // Authenticate user, return JWT or error
//     return "OK";
// }

pub async fn ping() -> impl IntoResponse {
    println!("pong");
    return "pong";
}
