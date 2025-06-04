use runelink_types::NewChannel;
use uuid::Uuid;

use crate::{error::CliError, requests, storage::TryGetDomainName, util::get_api_url};

use super::{
    config::{handle_default_channel_commands, DefaultChannelArgs},
    context::CliContext, select::get_server_selection,
};

#[derive(clap::Args, Debug)]
pub struct ChannelArgs {
    #[clap(subcommand)]
    pub command: ChannelCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum ChannelCommands {
    /// List all channels
    List(ChannelListArgs),
    /// Get a channel by ID
    Get(ChannelGetArgs),
    /// Create a new channel
    Create(ChannelCreateArgs),
    /// Manage default channels
    Default(DefaultChannelArgs),
}

#[derive(clap::Args, Debug)]
pub struct ChannelListArgs {
    /// Optional: Filter channels by Server ID
    #[clap(long)]
    pub server_id: Option<Uuid>,
    /// Include channels from all servers you are a member of
    #[clap(short, long)]
    pub all: bool,
}

#[derive(clap::Args, Debug)]
pub struct ChannelGetArgs {
    /// The ID of the channel
    #[clap(long)]
    pub channel_id: Uuid,
}

#[derive(clap::Args, Debug)]
pub struct ChannelCreateArgs {
    /// The title of the channel
    #[clap(long)]
    pub title: String,
    /// Optional: The description of the channel
    #[clap(long)]
    pub description: Option<String>,
    /// The server ID
    #[clap(long)]
    pub server_id: Uuid,
}

pub async fn handle_channel_commands(
    ctx: &mut CliContext<'_>,
    channel_args: &ChannelArgs,
) -> Result<(), CliError> {
    match &channel_args.command {
        ChannelCommands::List(list_args) => {
            let channels;
            if let Some(server_id) = list_args.server_id {
                let api_url = ctx.config.try_get_server_api_url(server_id)?;
                channels = requests::fetch_channels_by_server(
                    ctx.client, &api_url, server_id
                ).await?;
            } else if list_args.all {
                // TODO: only include servers the user is a member of
                let api_url = ctx.account.try_get_api_url()?;
                channels = requests::fetch_all_channels(
                    ctx.client, &api_url
                ).await?
                // Also, group by server for printing
            } else {
                let server = get_server_selection(ctx).await?;
                let api_url = get_api_url(&server.domain);
                channels = requests::fetch_channels_by_server(
                    ctx.client, &api_url, server.id
                ).await?
            }
            for channel in channels {
                println!("{} ({})", channel.title, channel.id);
            }
        },
        ChannelCommands::Get(get_args) => {
            let api_url = ctx.account.try_get_api_url()?;
            let channel = requests::fetch_channel_by_id(
                ctx.client, &api_url, get_args.channel_id
            ).await?;
            println!("{} ({})", channel.title, channel.id);
        },
        ChannelCommands::Create(create_args) => {
            let Some(account) = ctx.account else {
                return Err(CliError::MissingAccount);
            };
            let api_url = ctx.account.try_get_api_url()?;
            let new_channel = NewChannel {
                title: create_args.title.clone(),
                description: create_args.description.clone(),
            };
            let channel = requests::create_channel(
                ctx.client, &api_url, create_args.server_id, &new_channel
            ).await?;
            if let Some(server_config) =
                ctx.config.get_server_config_mut(channel.server_id)
            {
                if server_config.default_channel.is_none() {
                    server_config.default_channel = Some(channel.id);
                    ctx.config.save()?;
                }
            } else {
                let server = requests::fetch_server_by_id(
                    ctx.client, &api_url, channel.server_id
                ).await?;
                ctx.config.get_or_create_server_config(&server, &account.domain);
                ctx.config.save()?;
            }
            println!(
                "Created channel: {} ({}).",
                channel.title, channel.id
            );
        },
        ChannelCommands::Default(default_args) => {
            handle_default_channel_commands(ctx, default_args).await?;
        },
    };
    Ok(())
}
