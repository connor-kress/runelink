use crate::{
    cli::Cli,
    error::CliError,
    requests::do_ping,
};
use clap::{Parser};
use cli::handle_cli;
use reqwest::Client;
use storage::load_config;
// use storage::{load_config, save_config, AppConfig};

mod cli;
mod error;
mod requests;
mod storage;

fn get_api_url(domain: &str) -> String {
    format!("http://{}/api", domain)
}

#[allow(dead_code)]
async fn test_connectivities(client: &Client, domains: Vec<&str>) {
    println!("Hosts:");
    for domain in domains {
        let api_url = get_api_url(domain);
        match do_ping(client, &api_url).await {
            Ok(_) => println!("{} (ready)", domain),
            Err(_) => println!("{} (down)", domain),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), CliError> {
    // let config = load_config()?;
    // dbg!(config);
    let domain = "localhost:3000";

    let api_url = get_api_url(domain);

    let cli = Cli::parse();
    let client = Client::new();
    let mut config = load_config()?;
    handle_cli(&client, &cli, &api_url, &mut config).await?;

    Ok(())
}
