use std::process::ExitCode;

use clap::{Parser};
use cli::handle_cli;
use reqwest::{Client, StatusCode};
use storage::AppConfig;

use crate::{
    cli::Cli,
    error::CliError,
    requests::do_ping,
};

mod cli;
mod error;
mod requests;
mod storage;
mod util;

#[allow(dead_code)]
async fn test_connectivities(client: &Client, domains: Vec<&str>) {
    println!("Hosts:");
    for domain in domains {
        let api_url = util::get_api_url(domain);
        match do_ping(client, &api_url).await {
            Ok(_) => println!("{} (ready)", domain),
            Err(_) => println!("{} (down)", domain),
        }
    }
}

// sysexits.h inspired exit codes
const EX_OK: u8 = 0;
const EX_USAGE: u8 = 64; // command line usage error
const EX_DATAERR: u8 = 65; // data format error
const EX_NOUSER: u8 = 67; // addressee unknown (not found)
const EX_UNAVAILABLE: u8 = 69; // service unavailable
const EX_SOFTWARE: u8 = 70; // internal software error
const EX_IOERR: u8 = 74; // input/output error
const EX_TEMPFAIL: u8 = 75; // temp failure; user is invited to retry
const EX_PROTOCOL: u8 = 76; // remote error in protocol
const EX_NOPERM: u8 = 77; // permission denied
const EX_CONFIG: u8 = 78; // configuration error

fn status_to_exit_code(status: StatusCode) -> u8 {
    if status.is_client_error() { // 4xx
        match status.as_u16() {
            400 => EX_USAGE,     // Bad Request
            401 => EX_NOPERM,    // Unauthorized
            403 => EX_NOPERM,    // Forbidden
            404 => EX_NOUSER,    // Not Found (can mean resource or user)
            408 => EX_TEMPFAIL,  // Request Timeout (client-side)
            409 => EX_DATAERR,   // Conflict (data state issue)
            422 => EX_DATAERR,   // Unprocessable Entity (validation error)
            429 => EX_TEMPFAIL,  // Too Many Requests (rate limiting)
            _ => EX_DATAERR,     // Other 4xx client errors
        }
    } else if status.is_server_error() { // 5xx
        match status.as_u16() {
            500 => EX_SOFTWARE,    // Internal Server Error
            501 => EX_UNAVAILABLE, // Not Implemented
            502 => EX_UNAVAILABLE, // Bad Gateway
            503 => EX_UNAVAILABLE, // Service Unavailable
            504 => EX_TEMPFAIL,    // Gateway Timeout
            _ => EX_UNAVAILABLE,   // Other 5xx server errors
        }
    } else {
        // Non-error status, treat as a general software error
        EX_SOFTWARE
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    async fn run_app() -> Result<(), CliError> {
        let mut config = AppConfig::load()?;
        let cli = Cli::parse();
        let client = Client::new();
        handle_cli(&client, &cli, &mut config).await
    }

    let Err(cli_error) = run_app().await else {
        // Early return for success
        return ExitCode::from(EX_OK);
    };

    // Print error
    match &cli_error {
        CliError::ReqwestError(e) => {
            if let Some(status) = e.status() {
                eprintln!("{}: {}", status, e.to_string());
            } else {
                eprintln!("{}", e.to_string());
            }
        },
        CliError::InvalidArgument(msg) => eprintln!("{}", msg),
        other_error => eprintln!("{}", other_error),
    }

    // Return exit code
    ExitCode::from(match cli_error {
        CliError::ReqwestError(e) => {
            if let Some(status) = e.status() {
                status_to_exit_code(status)
            } else if e.is_timeout() || e.is_connect() {
                EX_TEMPFAIL
            } else if e.is_request() {
                EX_USAGE
            } else {
                EX_PROTOCOL
            }
        },
        CliError::ApiStatusError { status, .. } => status_to_exit_code(status),
        CliError::InvalidArgument(_) => EX_USAGE,
        CliError::JsonDeserializeError(_) => EX_DATAERR,
        CliError::IoError(_) => EX_IOERR,
        CliError::UuidError(_) => EX_DATAERR,
        CliError::ConfigError(_) => EX_CONFIG,
        CliError::MissingAccount => EX_USAGE,
        CliError::Unknown(_) => EX_SOFTWARE,
    })
}
