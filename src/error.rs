use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
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

impl From<DieselError> for ApiError {
    fn from(e: DieselError) -> Self {
        match e {
            DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                _,
            ) => ApiError::UniqueViolation,
            DieselError::NotFound => ApiError::NotFound,
            DieselError::DatabaseError(_, info) => {
                ApiError::DatabaseError(info.message().to_string())
            }
            _ => ApiError::Unknown(format!("{:?}", e)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
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
        }
        .into_response()
    }
}

impl From<JoinError> for ApiError {
    fn from(e: JoinError) -> Self {
        ApiError::Unknown(format!("Join error: {e}"))
    }
}
