use runelink_client::requests;
use uuid::Uuid;

use crate::error::CliError;

use super::context::CliContext;

#[derive(clap::Args, Debug)]
pub struct UserArgs {
    #[clap(subcommand)]
    pub command: UserCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum UserCommands {
    /// List all users
    List(UserListArgs),
    /// Get a user by ID
    Get(UserGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct UserGetArgs {
    /// The domain of the user
    #[clap(long)]
    pub domain: Option<String>,
    /// The ID of the user to fetch
    #[clap(long)]
    pub user_id: Uuid,
}

#[derive(clap::Args, Debug)]
pub struct UserListArgs {
    /// The domain of the host
    #[clap(long)]
    pub domain: Option<String>,
    /// The ID of the server
    #[clap(long)]
    pub server_id: Option<Uuid>,
}

pub async fn handle_user_commands(
    ctx: &mut CliContext<'_>,
    user_args: &UserArgs,
) -> Result<(), CliError> {
    match &user_args.command {
        UserCommands::List(list_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;

            let users;
            if let Some(_server_id) = list_args.server_id {
                todo!(
                    "Missing endpoint: list users by server via Home Server (requires federated/proxy support)"
                );
            } else if let Some(domain) = &list_args.domain {
                // In the new model, clients never talk to remote servers directly.
                // Listing users from a remote server requires a Home Server proxy/federation endpoint.
                if domain != &account.domain {
                    todo!(
                        "Missing endpoint: list users from remote domain via Home Server (federated/proxy)"
                    );
                }
                users =
                    requests::users::fetch_all(ctx.client, &api_url).await?;
            } else {
                users =
                    requests::users::fetch_all(ctx.client, &api_url).await?;
            }
            for user in users {
                println!("{}", user.verbose());
            }
        }

        UserCommands::Get(get_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;

            if let Some(domain) = &get_args.domain {
                // If they explicitly ask for a remote domain, we need a federated lookup.
                if domain != &account.domain {
                    todo!(
                        "Missing endpoint: fetch user by id from remote domain via Home Server (federated/proxy)"
                    );
                }
            }
            let user = requests::users::fetch_by_id(
                ctx.client,
                &api_url,
                get_args.user_id,
            )
            .await?;
            println!("{}", user.verbose());
        }
    }
    Ok(())
}
