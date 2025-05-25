use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "rune")]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage users
    Users(UsersArgs),
    /// Manage messages
    Messages(MessagesArgs),
    /// Generate shell completion scripts
    Completions(CompletionsArgs), // New command
}

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    #[clap(value_parser = clap::value_parser!(Shell))]
    pub shell: Shell,
}

#[derive(Args, Debug)]
pub struct UsersArgs {
    #[clap(subcommand)]
    pub command: UsersCommands,
}

#[derive(Subcommand, Debug)]
pub enum UsersCommands {
    /// List all users
    List,
    /// Get a specific user by ID
    Get(GetUserArgs),
}

#[derive(Args, Debug)]
pub struct GetUserArgs {
    /// The ID of the user to fetch
    #[clap(long)]
    pub user_id: Uuid,
}

#[derive(Args, Debug)]
pub struct MessagesArgs {
    #[clap(subcommand)]
    pub command: MessagesCommands,
}

#[derive(Subcommand, Debug)]
pub enum MessagesCommands {
    /// List all messages (TODO)
    List,
}
