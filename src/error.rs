use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::error::Error as SqlxError;
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

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<SqlxError> for ApiError {
    fn from(e: SqlxError) -> Self {
        match e {
            SqlxError::PoolTimedOut | SqlxError::PoolClosed => {
                ApiError::DbConnectionError(e.to_string())
            }

            SqlxError::Database(db_err) => match db_err.code().as_deref() {
                Some("23505") => ApiError::UniqueViolation,
                _ => ApiError::DatabaseError(db_err.message().to_string()),
            },

            SqlxError::RowNotFound => ApiError::NotFound,

            _ => ApiError::Unknown(e.to_string()),
        }
    }
}

impl From<JoinError> for ApiError {
    fn from(e: JoinError) -> Self {
        ApiError::Unknown(format!("Join error: {e}"))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::DbConnectionError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            ApiError::UniqueViolation => {
                (StatusCode::CONFLICT, self.to_string())
            }
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::DatabaseError(_) | ApiError::Unknown(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };
        (status, body).into_response()
    }
}
