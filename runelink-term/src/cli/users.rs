use uuid::Uuid;

use crate::{error::CliError, requests, storage::TryGetDomainName};

use super::context::CliContext;

#[derive(clap::Args, Debug)]
pub struct UserArgs {
    #[clap(subcommand)]
    pub command: UserCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum UserCommands {
    /// List all users
    List,
    /// Get a user by ID
    Get(UserGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct UserGetArgs {
    /// The ID of the user to fetch
    #[clap(long)]
    pub user_id: Uuid,
}

pub async fn handle_user_commands(
    ctx: &mut CliContext<'_>,
    user_args: &UserArgs,
) -> Result<(), CliError> {
    match &user_args.command {
        UserCommands::List => {
            let api_url = ctx.account.try_get_api_url()?;
            let users = requests::fetch_users(ctx.client, &api_url).await?;
            for user in users {
                println!("{}@{} ({})", user.name, user.domain, user.id);
            }
        }
        UserCommands::Get(get_args) => {
            let api_url = ctx.account.try_get_api_url()?;
            let user = requests::fetch_user_by_id(
                ctx.client, &api_url, get_args.user_id
            ).await?;
            println!("{}@{} ({})", user.name, user.domain, user.id);
        }
    }
    Ok(())
}
