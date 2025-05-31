use uuid::Uuid;

use crate::{error::CliError, requests, storage::TryGetDomainName};

use super::context::CliContext;

#[derive(clap::Args, Debug)]
pub struct MessageArgs {
    #[clap(subcommand)]
    pub command: MessageCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum MessageCommands {
    /// List all messages
    List(MessageListArgs),
    /// Get a message by ID
    Get(MessageGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct MessageListArgs {
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
    ctx: &mut CliContext<'_>,
    message_args: &MessageArgs,
) -> Result<(), CliError> {
    match &message_args.command {
        MessageCommands::List(list_args) => {
            let api_url = ctx.account.try_get_api_url()?;
            let messages;
            if let Some(channel_id) = list_args.channel_id {
                messages = requests::fetch_messages_by_channel(
                    ctx.client, &api_url, channel_id
                ).await?;
            } else if let Some(server_id) = list_args.server_id {
                messages = requests::fetch_messages_by_server(
                    ctx.client, &api_url, server_id
                ).await?;
            } else {
                messages = requests::fetch_all_messages(
                    ctx.client, &api_url
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
        MessageCommands::Get(get_args) => {
            let api_url = ctx.account.try_get_api_url()?;
            let message = requests::fetch_message_by_id(
                ctx.client, &api_url, get_args.message_id
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
