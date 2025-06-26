use runelink_client::{requests, util::get_api_url};
use runelink_types::NewMessage;
use uuid::Uuid;

use crate::error::CliError;

use super::{
    context::CliContext, domain_query::DomainQueryBuilder,
    input::unwrap_or_prompt, select::get_channel_selection_with_inputs,
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
            ctx.account.ok_or(CliError::MissingAccount)?;
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
                println!("{}", message);
            }
        }

        MessageCommands::Get(get_args) => {
            ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = DomainQueryBuilder::new(ctx)
                .try_domain(get_args.domain.clone())
                .try_server(get_args.server_id)
                .get_api_url()?;
            let message = requests::fetch_message_by_id(
                ctx.client, &api_url, get_args.message_id
            ).await?;
            println!("{}", message);
        }

        MessageCommands::Send(send_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let (server, channel) = get_channel_selection_with_inputs(
                ctx,
                send_args.channel_id,
                send_args.server_id,
            ).await?;
            let body = unwrap_or_prompt(send_args.body.clone(), "Message")?;
            let server_api_url = get_api_url(&server.domain);
            let new_message = NewMessage {
                body,
                author_id: account.user_id,
            };
            let message = requests::send_message(
                ctx.client, &server_api_url, channel.id, &new_message
            ).await?;
            println!("Sent message: {}", message.body);
        }
    };
    Ok(())
}
