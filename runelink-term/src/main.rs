use crate::{
    cli::Cli,
    error::CliError,
    requests::{do_ping, fetch_users},
};
use clap::{CommandFactory, Parser};
use cli::{Commands, MessagesCommands, UsersCommands};
use requests::{fetch_all_messages, fetch_message_by_id, fetch_messages_by_channel, fetch_messages_by_server, fetch_user_by_id};
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
                    println!("{}@{}", user.name, user.domain);
                }
            }
            UsersCommands::Get(get_args) => {
                let user = fetch_user_by_id(
                    &client, &api_url,
                    get_args.user_id,
                ).await?;
                println!("{}@{}", user.name, user.domain);
            }
        },
        Commands::Messages(messages_args) => match &messages_args.command {
            MessagesCommands::List(list_args) => {
                let messages;
                if let Some(channel_id) = list_args.channel_id {
                    messages = fetch_messages_by_channel(
                        &client, &api_url, channel_id
                    ).await?;
                } else if let Some(server_id) = list_args.server_id {
                    messages = fetch_messages_by_server(
                        &client, &api_url, server_id
                    ).await?;
                } else {
                    messages = fetch_all_messages(&client, &api_url).await?
                }
                for message in messages {
                    let author_name = message
                        .author
                        .map(|u| u.name)
                        .unwrap_or("Anon".into());
                    println!("{}: {}", author_name, message.body);
                }
            }
            MessagesCommands::Get(get_args) => {
                let message = fetch_message_by_id(
                    &client, &api_url,
                    get_args.message_id,
                ).await?;
                let author_name = message
                    .author
                    .map(|u| u.name)
                    .unwrap_or("Anon".into());
                println!("{}: {}", author_name, message.body);
            }
        },
        Commands::Completions(args) => {
            let mut cmd = Cli::command();
            let cmd_name = cmd.get_name().to_string();
            clap_complete::generate(
                args.shell, &mut cmd,
                cmd_name, &mut std::io::stdout(),
            );
        }
    }

    Ok(())
}
