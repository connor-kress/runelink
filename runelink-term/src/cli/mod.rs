use crate::error::CliError;
use clap::CommandFactory;
use clap_complete::Shell;
use messages::{handle_message_commands, MessagesArgs};
use reqwest::Client;
use users::{handle_user_commands, UsersArgs};

pub mod messages;
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
    /// Manage users
    Users(UsersArgs),
    /// Manage messages
    Messages(MessagesArgs),
    /// Generate shell completion scripts
    Completions(CompletionsArgs),
}

#[derive(clap::Args, Debug)]
pub struct CompletionsArgs {
    #[clap(value_parser = clap::value_parser!(Shell))]
    pub shell: Shell,
}

pub async fn handle_cli(
    client: &Client, cli: &Cli, api_url: &str
) -> Result<(), CliError> {
    match &cli.command {
        Commands::Users(users_args) => {
            handle_user_commands(client, api_url, users_args).await?
        },
        Commands::Messages(messages_args) => {
            handle_message_commands(client, api_url, messages_args).await?
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
