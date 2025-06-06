use runelink_client::{requests, util::get_api_url};
use runelink_types::NewMessage;
use uuid::Uuid;

use crate::{error::CliError, storage::TryGetDomain};

use super::{
    context::CliContext,
    input::read_input,
    select::get_channel_selection_with_inputs,
};

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
    /// Send a message
    Send(MessageSendArgs),
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
    /// The ID of the server the message is in
    #[clap(long)]
    pub server_id: Option<Uuid>,
    /// The domain of the server the message is in
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct MessageSendArgs {
    /// The body of the message
    #[clap(long)]
    pub body: Option<String>,
    /// The server ID
    #[clap(long)]
    pub server_id: Option<Uuid>,
    /// The channel ID
    #[clap(long)]
    pub channel_id: Option<Uuid>,
    /// The domain of the server
    #[clap(long)]
    pub domain: Option<String>,
}

pub async fn handle_message_commands(
    ctx: &mut CliContext<'_>,
    message_args: &MessageArgs,
) -> Result<(), CliError> {
    match &message_args.command {
        MessageCommands::List(list_args) => {
            // TODO: add cli arguments for each of these cases
            // let messages;
            // if let Some(channel_id) = list_args.channel_id {
            //     messages = requests::fetch_messages_by_channel(
            //         ctx.client, &api_url, channel_id
            //     ).await?;
            // } else if let Some(server_id) = list_args.server_id {
            //     messages = requests::fetch_messages_by_server(
            //         ctx.client, &api_url, server_id
            //     ).await?;
            // } else {
            //     messages = requests::fetch_all_messages(
            //         ctx.client, &api_url
            //     ).await?
            // }
            let (server, channel) = get_channel_selection_with_inputs(
                ctx,
                list_args.channel_id,
                list_args.server_id,
            ).await?;
            let api_url = get_api_url(&server.domain);
            let messages = requests::fetch_messages_by_channel(
                ctx.client,
                &api_url,
                channel.id,
            ).await?;
            for message in messages.iter().rev() {
                let author_name = message
                    .author
                    .as_ref()
                    .map(|u| u.name.as_str())
                    .unwrap_or("Anon");
                println!("{}: {}", author_name, message.body);
            }
        },

        MessageCommands::Get(get_args) => {
            let api_url = if let Some(domain) = &get_args.domain {
                get_api_url(domain)
            } else if let Some(server_id) = get_args.server_id {
                ctx.config
                    .try_get_server_api_url(server_id)
                    .unwrap_or(ctx.account.try_get_api_url()?)
            } else {
                ctx.account.try_get_api_url()?
            };
            let message = requests::fetch_message_by_id(
                ctx.client, &api_url, get_args.message_id
            ).await?;
            let author_name = message
                .author
                .map(|u| u.name)
                .unwrap_or("Anon".into());
            println!("{}: {}", author_name, message.body);
        },

        MessageCommands::Send(send_args) => {
            let Some(account) = ctx.account else {
                return Err(CliError::MissingAccount);
            };
            let (server, channel) = get_channel_selection_with_inputs(
                ctx,
                send_args.channel_id,
                send_args.server_id,
            ).await?;
            let body = if let Some(body) = &send_args.body {
                body.clone()
            } else {
                read_input("Message: ")?
                    .ok_or_else(|| CliError::InvalidArgument(
                        "Message body is required.".into()
                    ))?
            };
            let server_api_url = get_api_url(&server.domain);
            let new_message = NewMessage {
                body,
                author_id: account.user_id,
            };
            let message = requests::send_message(
                ctx.client, &server_api_url, channel.id, &new_message
            ).await?;
            println!("Sent message: {}", message.body);
        },
    };
    Ok(())
}
