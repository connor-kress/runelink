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
    /// Manage default server
    Default(DefaultServerArgs),
}

#[derive(clap::Args, Debug)]
pub struct ServerIdArg {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Uuid,
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
        ServerCommands::Default(default_args) => {
            handle_default_server_commands(ctx, &default_args).await?;
        }
    }
    Ok(())
}
