use crate::{
    cli::Cli,
    error::CliError,
    requests::{do_ping, fetch_users},
};
use clap::Parser;
use cli::{Commands, MessagesCommands, UsersCommands};
use requests::fetch_user_by_id;
use reqwest::Client;

mod cli;
mod error;
mod requests;

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
    let cli = Cli::parse();
    let client = Client::new();

    let domain = "localhost:3000";
    // let bad_domain = "localhost:9999";
    // test_connectivities(&client, vec![domain, bad_domain]).await;
    // println!();

    let api_url = get_api_url(domain);

    match &cli.command {
        Commands::Users(users_args) => match &users_args.command {
            UsersCommands::List => {
                let users = fetch_users(&client, &api_url).await?;
                for user in users {
                    println!("{:?}", user);
                }
            }
            UsersCommands::Get(get_args) => {
                let user = fetch_user_by_id(
                    &client,
                    &api_url,
                    get_args.user_id,
                ).await?;
                println!("{:?}", user);
            }
        },
        Commands::Messages(messages_args) => match &messages_args.command {
            MessagesCommands::List => {
                todo!();
            }
        },
    }

    Ok(())
}
