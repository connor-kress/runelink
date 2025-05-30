use crate::{error::CliError, storage::AppConfig};
use channels::{handle_channel_commands, ChannelArgs};
use clap::CommandFactory;
use clap_complete::Shell;
use reqwest::Client;
use messages::{handle_message_commands, MessageArgs};
use servers::{handle_server_commands, ServerArgs};
use users::{handle_user_commands, UserArgs};

pub mod channels;
pub mod config;
pub mod messages;
pub mod servers;
pub mod users;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "rune")]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Manage channels
    Channels(ChannelArgs),
    /// Manage messages
    Messages(MessageArgs),
    /// Manage servers
    Servers(ServerArgs),
    /// Manage users
    Users(UserArgs),
    /// Generate shell completion scripts
    Completions(CompletionsArgs),
}

#[derive(clap::Args, Debug)]
pub struct CompletionsArgs {
    #[clap(value_parser = clap::value_parser!(Shell))]
    pub shell: Shell,
}

pub async fn handle_cli(
    client: &Client,
    cli: &Cli,
    api_url: &str,
    config: &mut AppConfig,
) -> Result<(), CliError> {
    match &cli.command {
        Commands::Channels(channel_args) => {
            handle_channel_commands(client, api_url, config, channel_args).await?
        },
        Commands::Messages(message_args) => {
            handle_message_commands(client, api_url, config, message_args).await?
        },
        Commands::Servers(server_args) => {
            handle_server_commands(client, api_url, config, server_args).await?
        },
        Commands::Users(user_args) => {
            handle_user_commands(client, api_url, config, user_args).await?
        },
        Commands::Completions(args) => {
            let mut cmd = Cli::command();
            let cmd_name = cmd.get_name().to_string();
            clap_complete::generate(
                args.shell, &mut cmd,
                cmd_name, &mut std::io::stdout(),
            );
        }
    }
    Ok(())
}
