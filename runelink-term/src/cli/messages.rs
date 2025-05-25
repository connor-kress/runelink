use crate::{error::CliError, requests};
use reqwest::Client;
use uuid::Uuid;

#[derive(clap::Args, Debug)]
pub struct MessagesArgs {
    #[clap(subcommand)]
    pub command: MessagesCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum MessagesCommands {
    /// List messages
    List(MessagesListArgs),
    Get(MessageGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct MessagesListArgs {
    /// Optional: Filter messages by Server ID
    #[clap(long)]
    pub server_id: Option<Uuid>,

    /// Optional: Filter messages by Channel ID
    #[clap(long)]
    pub channel_id: Option<Uuid>,
}

#[derive(clap::Args, Debug)]
pub struct MessageGetArgs {
    /// The ID of the message to fetch
    #[clap(long)]
    pub message_id: Uuid,
}

pub async fn handle_message_commands(
    client: &Client, api_url: &str, messages_args: &MessagesArgs
) -> Result<(), CliError> {
    match &messages_args.command {
        MessagesCommands::List(list_args) => {
            let messages;
            if let Some(channel_id) = list_args.channel_id {
                messages = requests::fetch_messages_by_channel(
                    &client, &api_url, channel_id
                ).await?;
            } else if let Some(server_id) = list_args.server_id {
                messages = requests::fetch_messages_by_server(
                    &client, &api_url, server_id
                ).await?;
            } else {
                messages = requests::fetch_all_messages(
                    &client, &api_url
                ).await?
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
            let message = requests::fetch_message_by_id(
                &client, &api_url,
                get_args.message_id,
            ).await?;
            let author_name = message
                .author
                .map(|u| u.name)
                .unwrap_or("Anon".into());
            println!("{}: {}", author_name, message.body);
        }
    };
    Ok(())
}
