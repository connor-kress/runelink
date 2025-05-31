use runelink_types::NewServer;
use uuid::Uuid;

use crate::{
    error::CliError,
    requests,
    storage::TryGetDomainName,
};

use super::{
    config::{handle_default_server_commands, DefaultServerArgs},
    context::CliContext,
};

#[derive(clap::Args, Debug)]
pub struct ServerArgs {
    #[clap(subcommand)]
    pub command: ServerCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum ServerCommands {
    /// List all servers
    List,
    /// Get a server by ID
    Get(ServerIdArg),
    /// Create a new server
    Create(NewServerArgs),
    /// Manage default server
    Default(DefaultServerArgs),
}

#[derive(clap::Args, Debug)]
pub struct ServerIdArg {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Uuid,
}

#[derive(clap::Args, Debug)]
pub struct NewServerArgs {
    /// The title of the server
    #[clap(long)]
    pub title: String,
    /// Optional: The description of the server
    #[clap(long)]
    pub description: Option<String>,
}

pub async fn handle_server_commands(
    ctx: &mut CliContext<'_>,
    server_args: &ServerArgs,
) -> Result<(), CliError> {
    match &server_args.command {
        ServerCommands::List => {
            let api_url = ctx.account.try_get_api_url()?;
            let servers = requests::fetch_servers(ctx.client, &api_url).await?;
            for server in servers {
                println!("{} ({})", server.title, server.id);
            }
        }
        ServerCommands::Get(get_args) => {
            let api_url = ctx.account.try_get_api_url()?;
            let server = requests::fetch_server_by_id(
                ctx.client, &api_url, get_args.server_id
            ).await?;
            println!("{} ({})", server.title, server.id);
        }
        ServerCommands::Create(create_args) => {
            let Some(account) = ctx.account else {
                return Err(CliError::MissingAccount);
            };
            let api_url = ctx.account.try_get_api_url()?;
            let new_server = NewServer {
                title: create_args.title.clone(),
                description: create_args.description.clone(),
                user_id: account.user_id,
            };
            let server = requests::create_server(
                ctx.client, &api_url, &new_server
            ).await?;
            ctx.config.get_or_create_server_config(&server, &account.domain);
            ctx.config.save()?;
            println!(
                "Created server: {} ({}).",
                server.title, server.id
            );
        },
        ServerCommands::Default(default_args) => {
            handle_default_server_commands(ctx, &default_args).await?;
        }
    }
    Ok(())
}
