use crate::{error::CliError, requests};
use reqwest::Client;
use uuid::Uuid;

#[derive(clap::Args, Debug)]
pub struct ChannelArgs {
    #[clap(subcommand)]
    pub command: ChannelCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum ChannelCommands {
    /// List channels
    List(ChannelListArgs),
    Get(ChannelGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct ChannelListArgs {
    /// Optional: Filter channels by Server ID
    #[clap(long)]
    pub server_id: Option<Uuid>,
}

#[derive(clap::Args, Debug)]
pub struct ChannelGetArgs {
    /// The ID of the channel to fetch
    #[clap(long)]
    pub channel_id: Uuid,
}

pub async fn handle_channel_commands(
    client: &Client, api_url: &str, channel_args: &ChannelArgs
) -> Result<(), CliError> {
    match &channel_args.command {
        ChannelCommands::List(list_args) => {
            let channels;
            if let Some(server_id) = list_args.server_id {
                channels = requests::fetch_channels_by_server(
                    &client, &api_url, server_id
                ).await?;
            } else {
                channels = requests::fetch_all_channels(
                    &client, &api_url
                ).await?
            }
            for channel in channels {
                println!("{} - {}", channel.title, channel.id);
            }
        }
        ChannelCommands::Get(get_args) => {
            let channel = requests::fetch_channel_by_id(
                &client, &api_url,
                get_args.channel_id,
            ).await?;
            println!("{} - {}", channel.title, channel.id);
        }
    };
    Ok(())
}
