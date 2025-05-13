use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use diesel::result::{DatabaseErrorKind, Error as DieselError};

#[allow(dead_code)]
pub fn map_diesel_error_to_response(e: DieselError) -> Response {
    return map_diesel_error(e).into_response();
}

pub fn map_diesel_error(e: DieselError) -> (StatusCode, String) {
    return match e {
        DieselError::DatabaseError(kind, info) => match kind {
            DatabaseErrorKind::UniqueViolation => (
                StatusCode::CONFLICT,
                "Unique constraint violation".to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", info.message()),
            ),
        },
        DieselError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unknown database error: {:?}", e),
        ),
    };
}
