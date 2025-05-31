use clap::{Parser};
use cli::handle_cli;
use reqwest::Client;
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

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let mut config = AppConfig::load()?;
    let cli = Cli::parse();
    let client = Client::new();
    if let Err(cli_error) = handle_cli(&client, &cli, &mut config).await {
        match cli_error {
            CliError::ReqwestError(e) => {
                if let Some(status) = e.status() {
                    eprintln!("{}: {}", status, e.to_string());
                } else {
                    eprintln!("{}", e.to_string());
                }
            },
            _ => return Err(cli_error),
        }
    }

    Ok(())
}
