use reqwest::Client;
use runelink_types::NewUser;

use crate::{
    error::CliError,
    requests,
    storage::{save_config, AppConfig},
    util,
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
    client: &Client,
    config: &mut AppConfig,
    account_args: &AccountArgs,
) -> Result<(), CliError> {
    match &account_args.command {
        AccountCommands::List => {
            if config.accounts.is_empty() {
                println!("No accounts.");
                return Ok(());
            }
            for account in config.accounts.iter() {
                println!(
                    "{}@{} ({})",
                    account.name, account.domain, account.user_id
                );
            }

        },
        AccountCommands::Add(add_args) => {
            // TODO: switch for production
            // let api_url = util::get_api_url(&add_args.domain);
            let api_url = util::get_api_url("localhost:3000");
            let user = requests::fetch_user_by_name_and_domain(
                client,
                &api_url,
                add_args.name.clone(),
                add_args.domain.clone(),
            ).await?;
            config.get_or_create_account_config(&user);
            save_config(config)?;
            println!(
                "Added account: {}@{} ({}).",
                user.name, user.domain, user.id
            );
        },
        AccountCommands::Create(create_args) => {
            // TODO: switch for production
            // let api_url = util::get_api_url(&create_args.domain);
            let api_url = util::get_api_url("localhost:3000");
            let new_user = NewUser {
                name: create_args.name.clone(),
                domain: create_args.domain.clone(),
            };
            let user = requests::create_user(client, &api_url, &new_user).await?;
            config.get_or_create_account_config(&user);
            save_config(config)?;
            println!(
                "Created account: {}@{} ({}).",
                user.name, user.domain, user.id
            );
        },
    }
    Ok(())
}
