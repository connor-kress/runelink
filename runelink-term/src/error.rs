use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("API request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("API returned error status {status}: {message}")]
    ApiStatusError { status: StatusCode, message: String },

    #[error("Failed to parse JSON response: {0}")]
    JsonDeserializeError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid UUID: {0}")]
    UuidError(#[from] uuid::Error),

    #[allow(dead_code)]
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),

    #[error("Missing account: Specify an account or set a default account")]
    MissingAccount,

    #[allow(dead_code)]
    #[error("Unexpected error: {0}")]
    Unknown(String),
    // #[error("CLI argument parsing error: {0}")]
    // ClapError(#[from] clap::Error),
}
