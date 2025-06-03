use std::process::ExitCode;

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
async fn main() -> ExitCode {
    async fn run_app() -> Result<(), CliError> {
        crate::cli::select::demo_select_inline()?; // for testing
        let mut config = AppConfig::load()?;
        let cli = Cli::parse();
        let client = Client::new();
        handle_cli(&client, &cli, &mut config).await
    }

    match run_app().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(cli_error) => {
            cli_error.report_for_cli();
            cli_error.into()
        },
    }
}
