use runelink_client::requests;
use runelink_types::UserRef;
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
    pub domain: String,
    /// The ID of the user to fetch
    #[clap(long)]
    pub name: String,
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
            let api_url = ctx.home_api_url()?;
            let users;
            if let Some(server_id) = list_args.server_id {
                // Fetch members of the server, then extract users
                let members = requests::memberships::fetch_members_by_server(
                    ctx.client,
                    &api_url,
                    server_id,
                    list_args.domain.as_deref(),
                )
                .await?;
                users = members.into_iter().map(|m| m.user).collect();
            } else {
                users = requests::users::fetch_all(
                    ctx.client,
                    &api_url,
                    list_args.domain.as_deref(),
                )
                .await?;
            }
            for user in users {
                println!("{user}");
            }
        }

        UserCommands::Get(get_args) => {
            let user_ref =
                UserRef::new(get_args.name.clone(), get_args.domain.clone());
            let api_url = ctx.home_api_url()?;
            let user =
                requests::users::fetch_by_ref(ctx.client, &api_url, user_ref)
                    .await?;
            println!("{user}");
        }
    }
    Ok(())
}
