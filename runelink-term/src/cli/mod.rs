use clap::CommandFactory;
use clap_complete::Shell;
use context::CliContext;
use reqwest::Client;

use crate::{error::CliError, storage::AppConfig};

pub mod account;
pub mod channels;
pub mod config;
pub mod context;
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
    /// Optional: The account's username
    #[clap(long)]
    pub name: Option<String>,
    /// Optional: The domain name of the account's host
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Manage accounts
    Account(account::AccountArgs),
    /// Manage channels
    Channels(channels::ChannelArgs),
    /// Manage messages
    Messages(messages::MessageArgs),
    /// Manage servers
    Servers(servers::ServerArgs),
    /// Manage users
    Users(users::UserArgs),
    /// Manage config
    Config(config::ConfigArgs),
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
    config: &mut AppConfig,
) -> Result<(), CliError> {
    let account_owned = match (&cli.name, &cli.domain) {
        (Some(name), Some(domain)) => {
            config.get_account_config_by_name(name, domain)
        },
        _ => config.get_default_account(),
    }.map(|a| a.clone());
    let mut ctx_owned = CliContext {
        client,
        config,
        account: account_owned.as_ref(),
    };
    let ctx = &mut ctx_owned;

    match &cli.command {
        Commands::Account(args) => {
            account::handle_account_commands(ctx, args).await?;
        },
        Commands::Channels(args) => {
            channels::handle_channel_commands(ctx, args).await?;
        },
        Commands::Messages(args) => {
            messages::handle_message_commands(ctx, args).await?;
        },
        Commands::Servers(args) => {
            servers::handle_server_commands(ctx, args).await?;
        },
        Commands::Users(args) => {
            users::handle_user_commands(ctx, args).await?;
        },
        Commands::Config(args) => {
            config::handle_config_commands(ctx, args).await?;
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
