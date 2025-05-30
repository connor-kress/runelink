use reqwest::Client;
use uuid::Uuid;

use crate::{error::CliError, requests, storage::{save_config, AppConfig}};

#[derive(clap::Args, Debug)]
pub struct ConfigArgs {
    #[clap(subcommand)]
    pub command: ConfigCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum ConfigCommands {
    /// Manage default server
    DefaultServer(DefaultServerArgs),
    /// Manage default channels
    DefaultChannel(DefaultChannelArgs),
    /// Manage default host (temp)
    DefaultHost(DefaultHostArgs),
}

pub async fn handle_config_commands(
    client: &Client,
    api_url: &str,
    config: &mut AppConfig,
    config_args: &ConfigArgs,
) -> Result<(), CliError> {
    match &config_args.command {
        ConfigCommands::DefaultServer(default_server_args) => {
            handle_default_server_commands(
                client, api_url, config, default_server_args
            ).await?;
        },
        ConfigCommands::DefaultChannel(default_channel_args) => {
            handle_default_channel_commands(
                client, api_url, config, default_channel_args
            ).await?;
        },
        ConfigCommands::DefaultHost(default_host_args) => {
            handle_default_host_commands(config, default_host_args).await?;
        },
    }
    Ok(())
}

// DEFAULT HOST

#[derive(clap::Args, Debug)]
pub struct DefaultHostArgs {
    #[clap(subcommand)]
    pub command: DefaultHostCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum DefaultHostCommands {
    /// Show the default host
    Get,
    /// Set the default host
    Set(DomainNameArg),
}

#[derive(clap::Args, Debug)]
pub struct DomainNameArg {
    /// The domain name of the host
    #[clap(long)]
    pub domain: String,
}

pub async fn handle_default_host_commands(
    config: &mut AppConfig,
    default_args: &DefaultHostArgs,
) -> Result<(), CliError> {
    match &default_args.command {
        DefaultHostCommands::Get => {
            if let Some(domain_name) = config.default_host.clone() {
                println!("{}", domain_name);
            } else {
                println!("No default host set.");
            }
        }
        DefaultHostCommands::Set(set_default_args) => {
            config.default_host = Some(set_default_args.domain.clone());
            save_config(&config)?;
            println!("Set default host to '{}'", set_default_args.domain);
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
    client: &Client,
    api_url: &str,
    config: &mut AppConfig,
    default_args: &DefaultServerArgs,
) -> Result<(), CliError> {
    match &default_args.command {
        DefaultServerCommands::Get => {
            if let Some(server_id) = config.default_server {
                let server = requests::fetch_server_by_id(
                    client, api_url, server_id
                ).await?;
                println!("{} ({})", server.title, server.id);
            } else {
                println!("No default server set.");
            }
        }
        DefaultServerCommands::Set(set_default_args) => {
            let server = requests::fetch_server_by_id(
                client, api_url, set_default_args.server_id
            ).await?;
            config.default_server = Some(server.id);
            save_config(&config)?;
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
    /// The ID of the new default channel
    #[clap(long)]
    pub channel_id: Uuid,
}

pub async fn handle_default_channel_commands(
    client: &Client,
    api_url: &str,
    config: &mut AppConfig,
    default_args: &DefaultChannelArgs,
) -> Result<(), CliError> {
    match &default_args.command {
        DefaultChannelCommands::Get(get_default_args) => {
            if let Some(server_id) = get_default_args.server_id {
                let Some(server_config)
                    = config.get_server_config(server_id) else
                {
                    println!("No default channel set.");
                    return Ok(());
                };
                if let Some(channel_id) = server_config.default_channel {
                    let channel = requests::fetch_channel_by_id(
                        client, api_url, channel_id
                    ).await?;
                    println!("{} ({})", channel.title, channel.id);
                } else {
                    println!("No default channel set.");
                    return Ok(());
                }
                return Ok(());
            }
            if config.servers.is_empty() {
                println!("No default channels set.");
                return Ok(());
            }
            for server_config in config.servers.iter() {
                // TODO: endpoint for batch fetching servers/channels
                let server = requests::fetch_server_by_id(
                    client, api_url, server_config.server_id
                ).await?;
                println!("{} ({})", server.title, server.id);
                if let Some(channel_id) = server_config.default_channel {
                    let channel = requests::fetch_channel_by_id(
                        client, api_url, channel_id
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

        DefaultChannelCommands::Set(set_default_args) => {
            let server = requests::fetch_server_by_id(
                client, api_url, set_default_args.server_id
            ).await?;
            let server_channels = requests::fetch_channels_by_server(
                client, api_url, set_default_args.server_id
            ).await?;
            let channel = requests::fetch_channel_by_id(
                client, api_url, set_default_args.channel_id
            ).await?;
            if !server_channels.iter().any(|sc| sc.id == channel.id) {
                return Err(CliError::InvalidArgument(
                    "Channel must be in server.".into()
                ))
            }
            let server_config = config
                .get_or_create_server_config_mut(server.id);
            server_config.default_channel = Some(channel.id);
            save_config(&config)?;
        }
    }
    Ok(())
}
