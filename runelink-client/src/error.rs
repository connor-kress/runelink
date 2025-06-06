use reqwest::StatusCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("unexpected status {0}: {1}")]
    Status(StatusCode, String),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
