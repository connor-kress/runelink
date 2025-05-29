use crate::{
    error::CliError,
    requests,
    storage::{save_config, AppConfig},
};
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
    Get(ServerIdArg),
    /// Get a specific server by ID
    Default(DefaultServerArgs),
}

#[derive(clap::Args, Debug)]
pub struct DefaultServerArgs {
    #[clap(subcommand)]
    command: DefaultServerCommands,
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

pub async fn handle_server_commands(
    client: &Client,
    api_url: &str,
    config: &mut AppConfig,
    server_args: &ServerArgs,
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
        ServerCommands::Default(default_args) => match &default_args.command {
            DefaultServerCommands::Get => {
                if let Some(server_id) = config.default_server {
                    let server = requests::fetch_server_by_id(
                        client, api_url, server_id
                    ).await?;
                    println!("{} - {}", server.title, server.id);
                } else {
                    println!("None");
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
    }
    Ok(())
}
