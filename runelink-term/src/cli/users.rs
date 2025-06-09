use runelink_client::requests;
use uuid::Uuid;

use crate::error::CliError;

use super::{context::CliContext, domain_query::DomainQueryBuilder};

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
            ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = DomainQueryBuilder::new(ctx)
                .try_domain(list_args.domain.clone())
                .try_server(list_args.server_id)
                .get_api_url()?;
            let users;
            if let Some(_server_id) = list_args.server_id {
                todo!("fetch users by server");
            } else {
                users = requests::fetch_users(ctx.client, &api_url).await?;
            }
            for user in users {
                println!("{}@{} ({})", user.name, user.domain, user.id);
            }
        }
        UserCommands::Get(get_args) => {
            ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = DomainQueryBuilder::new(ctx)
                .try_domain(get_args.domain.clone())
                .get_api_url()?;
            let user = requests::fetch_user_by_id(
                ctx.client, &api_url, get_args.user_id
            ).await?;
            println!("{}@{} ({})", user.name, user.domain, user.id);
        }
    }
    Ok(())
}
