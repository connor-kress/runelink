use runelink_client::{requests, util::get_api_url};
use uuid::Uuid;

use crate::{error::CliError, storage::TryGetDomain};

use super::context::CliContext;

#[derive(clap::Args, Debug)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub command: ConfigCommands,
}

#[allow(clippy::enum_variant_names)]
#[derive(clap::Subcommand, Debug)]
pub enum ConfigCommands {
    /// Manage default account
    DefaultAccount(DefaultAccountArgs),
    /// Manage default server
    DefaultServer(DefaultServerArgs),
    /// Manage default channels
    DefaultChannel(DefaultChannelArgs),
}

pub async fn handle_config_commands(
    ctx: &mut CliContext<'_>,
    config_args: &ConfigArgs,
) -> Result<(), CliError> {
    match &config_args.command {
        ConfigCommands::DefaultServer(default_server_args) => {
            handle_default_server_commands(ctx, default_server_args).await?;
        }
        ConfigCommands::DefaultChannel(default_channel_args) => {
            handle_default_channel_commands(ctx, default_channel_args).await?;
        }
        ConfigCommands::DefaultAccount(default_account_args) => {
            handle_default_account_commands(ctx, default_account_args).await?;
        }
    }
    Ok(())
}

// DEFAULT HOST

#[derive(clap::Args, Debug)]
pub struct DefaultAccountArgs {
    #[clap(subcommand)]
    pub command: DefaultAccountCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum DefaultAccountCommands {
    /// Show the default account
    Get,
    /// Set the default account
    Set(NameAndDomainArgs),
}

#[derive(clap::Args, Debug)]
pub struct NameAndDomainArgs {
    /// The account's username
    #[clap(long)]
    pub name: String,
    /// The domain name of the account's host
    #[clap(long)]
    pub domain: String,
}


pub async fn handle_default_account_commands(
    ctx: &mut CliContext<'_>,
    default_args: &DefaultAccountArgs,
) -> Result<(), CliError> {
    match &default_args.command {
        DefaultAccountCommands::Get => {
            if let Some(account) = ctx.config.get_default_account() {
                println!(
                    "{}@{} ({})",
                    account.name, account.domain, account.user_id
                );
            } else {
                println!("No default host set.");
            }
        }

        DefaultAccountCommands::Set(set_args) => {
            let account = ctx.config.get_account_config_by_name(
                &set_args.name,
                &set_args.domain,
            ).ok_or_else(|| {
                CliError::InvalidArgument("Account not found.".into())
            })?.clone();
            ctx.config.default_account = Some(account.user_id);
            ctx.config.save()?;
            println!(
                "Set default account: {}@{} ({}).",
                account.name, account.domain, account.user_id
            );
        }
    }
    Ok(())
}

// DEFAULT SERVER

#[derive(clap::Args, Debug)]
pub struct DefaultServerArgs {
    #[clap(subcommand)]
    pub command: DefaultServerCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum DefaultServerCommands {
    /// Show the default server
    Get,
    /// Set the default server
    Set(ServerIdArg),
}

#[derive(clap::Args, Debug)]
pub struct ServerIdArg {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Uuid,
}

pub async fn handle_default_server_commands(
    ctx: &mut CliContext<'_>,
    default_args: &DefaultServerArgs,
) -> Result<(), CliError> {
    match &default_args.command {
        DefaultServerCommands::Get => {
            if let Some(server_id) = ctx.config.default_server {
                let api_url = ctx.account.try_get_api_url()?;
                let server = requests::fetch_server_by_id(
                    ctx.client, &api_url, server_id
                ).await?;
                println!("{} ({})", server.title, server.id);
            } else {
                println!("No default server set.");
            }
        }

        DefaultServerCommands::Set(set_args) => {
            let api_url = ctx.account.try_get_api_url()?;
            let server = requests::fetch_server_by_id(
                ctx.client, &api_url, set_args.server_id
            ).await?;
            ctx.config.default_server = Some(server.id);
            ctx.config.save()?;
            println!("Set default server to '{}'.", server.title);
        }
    }
    Ok(())
}

// DEFAULT CHANNELS

#[derive(clap::Args, Debug)]
pub struct DefaultChannelArgs {
    #[clap(subcommand)]
    pub command: DefaultChannelCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum DefaultChannelCommands {
    /// Show default channels
    Get(ChannelGetDefaultArgs),
    /// Set a server's default channels
    Set(ChannelSetDefaultArgs),
}

#[derive(clap::Args, Debug)]
pub struct ChannelGetDefaultArgs {
    /// Optional: The ID of the server
    #[clap(long)]
    pub server_id: Option<Uuid>,
}

#[derive(clap::Args, Debug)]
pub struct ChannelSetDefaultArgs {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Uuid,
    /// The domain of the server
    #[clap(long)]
    pub server_domain: String,
    /// The ID of the new default channel
    #[clap(long)]
    pub channel_id: Uuid,
}

pub async fn handle_default_channel_commands(
    ctx: &mut CliContext<'_>,
    default_args: &DefaultChannelArgs,
) -> Result<(), CliError> {
    match &default_args.command {
        DefaultChannelCommands::Get(get_default_args) => {
            if let Some(server_id) = get_default_args.server_id {
                let Some(server_config) =
                    ctx.config.get_server_config(server_id) else
                {
                    println!("No default channel set.");
                    return Ok(());
                };
                if let Some(channel_id) = server_config.default_channel {
                    let api_url = ctx.account.try_get_api_url()?;
                    let channel = requests::fetch_channel_by_id(
                        ctx.client, &api_url, channel_id
                    ).await?;
                    println!("{} ({})", channel.title, channel.id);
                } else {
                    println!("No default channel set.");
                    return Ok(());
                }
                return Ok(());
            }
            if ctx.config.servers.is_empty() {
                println!("No default channels set.");
                return Ok(());
            }
            for server_config in ctx.config.servers.iter() {
                // TODO: endpoint for batch fetching servers/channels
                let api_url = get_api_url(&server_config.domain);
                let server = requests::fetch_server_by_id(
                    ctx.client, &api_url, server_config.server_id
                ).await?;
                println!("{} ({})", server.title, server.id);
                if let Some(channel_id) = server_config.default_channel {
                    let channel = requests::fetch_channel_by_id(
                        ctx.client, &api_url, channel_id
                    ).await?;
                    println!(
                        "\tDefault Channel: {} ({})",
                        channel.title, channel.id,
                    );
                } else {
                    println!("\tDefault Channel: None");
                }
            }
        }

        DefaultChannelCommands::Set(set_args) => {
            let api_url = get_api_url(&set_args.server_domain);
            let server = requests::fetch_server_by_id(
                ctx.client, &api_url, set_args.server_id
            ).await?;
            let server_channels = requests::fetch_channels_by_server(
                ctx.client, &api_url, set_args.server_id
            ).await?;
            let channel = requests::fetch_channel_by_id(
                ctx.client, &api_url, set_args.channel_id
            ).await?;
            if !server_channels.iter().any(|sc| sc.id == channel.id) {
                return Err(CliError::InvalidArgument(
                    "Channel not found in server.".into()
                ));
            }
            let server_config = ctx.config.get_or_create_server_config(
                &server,
                &set_args.server_domain,
            );
            server_config.default_channel = Some(channel.id);
            ctx.config.save()?;
            println!(
                "Set default channel to '{}' for '{}'.",
                channel.title, server.title
            );
        }
    }
    Ok(())
}
