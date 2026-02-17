use runelink_client::requests;
use runelink_types::NewMessage;
use uuid::Uuid;

use crate::error::CliError;

use super::{
    context::CliContext, input::unwrap_or_prompt,
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
    /// Delete a message
    Delete(MessageDeleteArgs),
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
    /// The ID of the channel the message is in
    #[clap(long)]
    pub channel_id: Uuid,
    /// The ID of the server the message is in
    #[clap(long)]
    pub server_id: Uuid,
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

#[derive(clap::Args, Debug)]
pub struct MessageDeleteArgs {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Uuid,
    /// The ID of the channel
    #[clap(long)]
    pub channel_id: Uuid,
    /// The ID of the message to delete
    #[clap(long)]
    pub message_id: Uuid,
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
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let (server, channel) = get_channel_selection_with_inputs(
                ctx,
                list_args.channel_id,
                list_args.server_id,
            )
            .await?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            let target_domain = if server.domain != account.user_ref.domain {
                Some(server.domain.as_str())
            } else {
                None
            };
            let messages = requests::messages::fetch_by_channel(
                ctx.client,
                &api_url,
                &access_token,
                server.id,
                channel.id,
                target_domain,
            )
            .await?;
            for message in messages.iter().rev() {
                println!("{message}");
            }
        }

        MessageCommands::Get(get_args) => {
            ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            let message = requests::messages::fetch_by_id(
                ctx.client,
                &api_url,
                &access_token,
                get_args.server_id,
                get_args.channel_id,
                get_args.message_id,
                get_args.domain.as_deref(),
            )
            .await?;
            println!("{message}");
        }

        MessageCommands::Send(send_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let (server, channel) = get_channel_selection_with_inputs(
                ctx,
                send_args.channel_id,
                send_args.server_id,
            )
            .await?;
            let body = unwrap_or_prompt(send_args.body.clone(), "Message")?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            let new_message = NewMessage {
                author: account.user_ref.clone(),
                body,
            };
            let target_domain = send_args.domain.as_deref().or_else(|| {
                if server.domain != account.user_ref.domain {
                    Some(server.domain.as_str())
                } else {
                    None
                }
            });
            let message = requests::messages::create(
                ctx.client,
                &api_url,
                &access_token,
                server.id,
                channel.id,
                &new_message,
                target_domain,
            )
            .await?;
            println!("Sent message: {}", message.body);
        }

        MessageCommands::Delete(delete_args) => {
            // TODO: Interactive message selection
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            requests::messages::delete(
                ctx.client,
                &api_url,
                &access_token,
                delete_args.server_id,
                delete_args.channel_id,
                delete_args.message_id,
                delete_args.domain.as_deref(),
            )
            .await?;
            println!("Deleted message: {}", delete_args.message_id);
        }
    };
    Ok(())
}
