use runelink_client::requests;
use runelink_types::NewChannel;
use uuid::Uuid;

use crate::error::CliError;

use super::{
    config::{DefaultChannelArgs, handle_default_channel_commands},
    context::CliContext,
    input::{read_input, unwrap_or_prompt},
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
    /// The domain of host or server
    #[clap(short, long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ChannelGetArgs {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Uuid,
    /// The domain of the server
    #[clap(long)]
    pub domain: Option<String>,
    /// The ID of the channel
    #[clap(long)]
    pub channel_id: Uuid,
}

#[derive(clap::Args, Debug)]
pub struct ChannelCreateArgs {
    /// The title of the channel
    #[clap(long)]
    pub title: Option<String>,
    /// The description of the channel
    #[clap(long)]
    pub description: Option<String>,
    /// Skip description cli prompt
    #[clap(long)]
    pub no_description: bool,
    /// The server ID
    #[clap(long)]
    pub server_id: Option<Uuid>,
}

pub async fn handle_channel_commands(
    ctx: &mut CliContext<'_>,
    channel_args: &ChannelArgs,
) -> Result<(), CliError> {
    match &channel_args.command {
        ChannelCommands::List(list_args) => {
            ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            let channels;
            if let Some(server_id) = list_args.server_id {
                channels = requests::channels::fetch_by_server(
                    ctx.client,
                    &api_url,
                    &access_token,
                    server_id,
                    None,
                )
                .await?;
            } else if list_args.all {
                channels = requests::channels::fetch_all(
                    ctx.client,
                    &api_url,
                    &access_token,
                    None,
                )
                .await?;
            } else {
                return Err(CliError::InvalidArgument(
                    "Server ID required. Use --server-id or --all.".into(),
                ));
            }
            if channels.is_empty() {
                println!(
                    "No channels available.\n\
                    For more information, try `rune channel --help`."
                )
            }
            for channel in channels {
                println!("{}", channel.verbose());
            }
        }

        ChannelCommands::Get(get_args) => {
            ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            let channel = requests::channels::fetch_by_id(
                ctx.client,
                &api_url,
                &access_token,
                get_args.server_id,
                get_args.channel_id,
                None,
            )
            .await?;
            println!("{}", channel.verbose());
        }

        ChannelCommands::Create(create_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            let server_id = create_args.server_id.ok_or_else(|| {
                CliError::InvalidArgument(
                    "Server ID required (--server-id)".into(),
                )
            })?;
            let server = requests::servers::fetch_by_id(
                ctx.client, &api_url, server_id, None,
            )
            .await?;
            let title =
                unwrap_or_prompt(create_args.title.clone(), "Channel Title")?;
            let desc = if create_args.description.is_some() {
                create_args.description.clone()
            } else if create_args.no_description {
                None
            } else {
                read_input("Channel Description (leave blank for none):\n> ")?
            };
            let new_channel = NewChannel {
                title,
                description: desc,
            };
            let channel = requests::channels::create(
                ctx.client,
                &api_url,
                &access_token,
                server_id,
                &new_channel,
                None,
            )
            .await?;
            if let Some(server_config) =
                ctx.config.get_server_config_mut(channel.server_id)
            {
                if server_config.default_channel.is_none() {
                    server_config.default_channel = Some(channel.id);
                    ctx.config.save()?;
                }
            } else {
                ctx.config
                    .get_or_create_server_config(&server, &account.domain);
                ctx.config.save()?;
            }
            println!("Created channel: {}", channel.verbose());
        }

        ChannelCommands::Default(default_args) => {
            handle_default_channel_commands(ctx, default_args).await?;
        }
    };
    Ok(())
}
