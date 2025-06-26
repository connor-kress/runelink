use runelink_client::{requests, util::get_api_url};
use runelink_types::NewUser;

use crate::{error::CliError, util};

use super::{
    config::{handle_default_account_commands, DefaultAccountArgs},
    context::CliContext,
};

#[derive(clap::Args, Debug)]
pub struct AccountArgs {
    #[clap(subcommand)]
    pub command: AccountCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum AccountCommands {
    /// List accounts
    List,
    /// Add an existing account
    Add(NameAndDomainArgs),
    /// Create a new account
    Create(NameAndDomainArgs),
    /// Manage default account
    Default(DefaultAccountArgs),
}

#[derive(clap::Args, Debug)]
pub struct NameAndDomainArgs {
    /// The account's username
    #[clap(long)]
    pub name: String,
    /// The domain name of the account's host
    #[clap(long)]
    pub domain: String,
}

pub async fn handle_account_commands(
    ctx: &mut CliContext<'_>,
    account_args: &AccountArgs,
) -> Result<(), CliError> {
    match &account_args.command {
        AccountCommands::List => {
            if ctx.config.accounts.is_empty() {
                println!("No accounts.");
                return Ok(());
            }
            for account in ctx.config.accounts.iter() {
                let prefix = util::get_prefix(
                    account.user_id,
                    ctx.config.default_account,
                    ctx.config.accounts.len(),
                );
                println!("{}{}", prefix, account.verbose());
            }

        },
        AccountCommands::Add(add_args) => {
            let api_url = get_api_url(&add_args.domain);
            let user = requests::fetch_user_by_name_and_domain(
                ctx.client,
                &api_url,
                add_args.name.clone(),
                add_args.domain.clone(),
            ).await?;
            ctx.config.get_or_create_account_config(&user);
            ctx.config.save()?;
            println!("Added account: {}.", user.verbose());
        },
        AccountCommands::Create(create_args) => {
            let api_url = get_api_url(&create_args.domain);
            let new_user = NewUser {
                name: create_args.name.clone(),
                domain: create_args.domain.clone(),
            };
            let user =
                requests::create_user(ctx.client, &api_url, &new_user).await?;
            ctx.config.get_or_create_account_config(&user);
            ctx.config.save()?;
            println!("Created account: {}.", user.verbose());
        },
        AccountCommands::Default(default_args) => {
            handle_default_account_commands(ctx, default_args).await?;
        },
    }
    Ok(())
}
