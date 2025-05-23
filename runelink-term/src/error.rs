use reqwest::StatusCode;
use std::io::Error as IoError;
use thiserror::Error;
use uuid::Error as UuidError;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("API request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("API returned error status {status}: {message}")]
    ApiStatusError { status: StatusCode, message: String },

    #[error("Failed to parse JSON response: {0}")]
    JsonDeserializeError(#[from] serde_json::Error),

    // #[error("CLI argument parsing error: {0}")]
    // ClapError(#[from] clap::Error),
    #[error("I/O error: {0}")]
    IoError(#[from] IoError),

    #[error("Invalid UUID: {0}")]
    UuidError(#[from] UuidError),

    #[allow(dead_code)]
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[allow(dead_code)]
    #[error("Unexpected error: {0}")]
    Unknown(String),
}
