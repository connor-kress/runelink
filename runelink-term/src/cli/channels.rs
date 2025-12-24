use runelink_client::{requests, util::get_api_url};
use runelink_types::NewChannel;
use uuid::Uuid;

use crate::error::CliError;

use super::{
    config::{DefaultChannelArgs, handle_default_channel_commands},
    context::CliContext,
    domain_query::DomainQueryBuilder,
    input::{read_input, unwrap_or_prompt},
    select::{ServerSelectionType, get_server_selection},
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
            let api_url = DomainQueryBuilder::new(ctx)
                .try_domain(list_args.domain.clone())
                .try_server(list_args.server_id)
                .get_api_url()?;
            let channels;
            if let Some(server_id) = list_args.server_id {
                channels = requests::fetch_channels_by_server(
                    ctx.client, &api_url, server_id,
                )
                .await?;
            } else if list_args.all {
                // TODO: only include servers the user is a member of
                // Also, fetch from multiple hosts (unless one is specified)
                channels =
                    requests::fetch_all_channels(ctx.client, &api_url).await?
                // Also, group by server for printing
            } else {
                let server =
                    get_server_selection(ctx, ServerSelectionType::MemberOnly)
                        .await?;
                let api_url = get_api_url(&server.domain);
                channels = requests::fetch_channels_by_server(
                    ctx.client, &api_url, server.id,
                )
                .await?;
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
            let api_url = DomainQueryBuilder::new(ctx)
                .try_domain(get_args.domain.clone())
                .try_server(Some(get_args.server_id))
                .get_api_url()?;
            let channel = requests::fetch_channel_by_id(
                ctx.client,
                &api_url,
                get_args.server_id,
                get_args.channel_id,
            )
            .await?;
            println!("{}", channel.verbose());
        }

        ChannelCommands::Create(create_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = DomainQueryBuilder::new(ctx)
                .try_server(create_args.server_id)
                .get_api_url()?;
            let server = if let Some(server_id) = create_args.server_id {
                requests::fetch_server_by_id(ctx.client, &api_url, server_id)
                    .await?
            } else {
                get_server_selection(ctx, ServerSelectionType::MemberOnly)
                    .await?
            };
            let title =
                unwrap_or_prompt(create_args.title.clone(), "Channel Title")?;
            let desc = if create_args.description.is_some() {
                create_args.description.clone()
            } else if create_args.no_description {
                None
            } else {
                read_input("Channel Description (leave blank for none):\n> ")?
            };
            let server_api_url = get_api_url(&server.domain);
            let new_channel = NewChannel {
                title,
                description: desc,
            };
            let channel = requests::create_channel(
                ctx.client,
                &server_api_url,
                server.id,
                &new_channel,
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
                let server = requests::fetch_server_by_id(
                    ctx.client,
                    &server_api_url,
                    channel.server_id,
                )
                .await?;
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
