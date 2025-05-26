use crate::{error::CliError, requests};
use reqwest::Client;
use uuid::Uuid;

#[derive(clap::Args, Debug)]
pub struct UserArgs {
    #[clap(subcommand)]
    pub command: UserCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum UserCommands {
    /// List all users
    List,
    /// Get a specific user by ID
    Get(UserGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct UserGetArgs {
    /// The ID of the user to fetch
    #[clap(long)]
    pub user_id: Uuid,
}

pub async fn handle_user_commands(
    client: &Client, api_url: &str, user_args: &UserArgs
) -> Result<(), CliError> {
    match &user_args.command {
        UserCommands::List => {
            let users = requests::fetch_users(&client, &api_url).await?;
            for user in users {
                println!("{}@{}", user.name, user.domain);
            }
        }
        UserCommands::Get(get_args) => {
            let user = requests::fetch_user_by_id(
                &client, &api_url,
                get_args.user_id,
            ).await?;
            println!("{}@{}", user.name, user.domain);
        }
    }
    Ok(())
}
