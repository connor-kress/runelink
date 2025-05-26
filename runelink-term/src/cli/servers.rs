use crate::{error::CliError, requests};
use reqwest::Client;
use uuid::Uuid;

#[derive(clap::Args, Debug)]
pub struct ServerArgs {
    #[clap(subcommand)]
    pub command: ServerCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum ServerCommands {
    /// List all servers
    List,
    /// Get a specific server by ID
    Get(ServerGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct ServerGetArgs {
    /// The ID of the server to fetch
    #[clap(long)]
    pub server_id: Uuid,
}

pub async fn handle_server_commands(
    client: &Client, api_url: &str, server_args: &ServerArgs
) -> Result<(), CliError> {
    match &server_args.command {
        ServerCommands::List => {
            let servers = requests::fetch_servers(&client, &api_url).await?;
            for server in servers {
                println!("{} - {}", server.title, server.id);
            }
        }
        ServerCommands::Get(get_args) => {
            let server = requests::fetch_server_by_id(
                &client, &api_url,
                get_args.server_id,
            ).await?;
            println!("{} - {}", server.title, server.id);
        }
    }
    Ok(())
}
