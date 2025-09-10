use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use runelink_client::Error as ClientError;
use serde::Serialize;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database connection error: {0}")]
    DbConnectionError(String),

    #[error("Unique constraint violation")]
    UniqueViolation,

    #[error("Resource not found")]
    NotFound,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Unauthorized: {0}")]
    AuthError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Upstream error: {0}")]
    Client(#[from] ClientError),
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => {
                ApiError::DbConnectionError(e.to_string())
            }

            sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
                Some("23505") => ApiError::UniqueViolation,
                _ => ApiError::DatabaseError(db_err.message().to_string()),
            },

            sqlx::Error::RowNotFound => ApiError::NotFound,

            _ => ApiError::Unknown(e.to_string()),
        }
    }
}

impl From<JoinError> for ApiError {
    fn from(e: JoinError) -> Self {
        ApiError::Unknown(format!("Join error: {e}"))
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::DbConnectionError(_)
            | ApiError::DatabaseError(_)
            | ApiError::Internal(_)
            | ApiError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::UniqueViolation => StatusCode::CONFLICT,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::AuthError(_) => StatusCode::UNAUTHORIZED,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Client(ref client_err) => match client_err {
                ClientError::Status(code, _) => *code,
                _ => StatusCode::BAD_GATEWAY,
            },
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}
