use uuid::Uuid;

use crate::{error::CliError, requests, storage::TryGetDomainName};

use super::{
    config::{handle_default_channel_commands, DefaultChannelArgs},
    context::CliContext,
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
    /// Manage default channels
    Default(DefaultChannelArgs),
}

#[derive(clap::Args, Debug)]
pub struct ChannelListArgs {
    /// Optional: Filter channels by Server ID
    #[clap(long)]
    pub server_id: Option<Uuid>,
}

#[derive(clap::Args, Debug)]
pub struct ChannelGetArgs {
    /// The ID of the channel
    #[clap(long)]
    pub channel_id: Uuid,
}


pub async fn handle_channel_commands(
    ctx: &mut CliContext<'_>,
    channel_args: &ChannelArgs,
) -> Result<(), CliError> {
    match &channel_args.command {
        ChannelCommands::List(list_args) => {
            let api_url = ctx.account.try_get_api_url()?;
            let channels;
            if let Some(server_id) = list_args.server_id {
                channels = requests::fetch_channels_by_server(
                    ctx.client, &api_url, server_id
                ).await?;
            } else {
                channels = requests::fetch_all_channels(
                    ctx.client, &api_url
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
        ChannelCommands::Default(default_args) => {
            handle_default_channel_commands(ctx, default_args).await?;
        },
    };
    Ok(())
}
