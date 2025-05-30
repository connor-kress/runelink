use reqwest::Client;
use uuid::Uuid;

use crate::{error::CliError, requests, storage::{save_config, AppConfig}, util};

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
    Add(AccountAddArgs),
}

#[derive(clap::Args, Debug)]
pub struct AccountAddArgs {
    /// The user ID (TODO: use name instead)
    #[clap(long)]
    pub user_id: Uuid,
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
                // TODO: switch for production
                // let api_url = util::get_api_url(&account.domain);
                let api_url = util::get_api_url("localhost:3000");
                let user = requests::fetch_user_by_id(
                    client, &api_url, account.user_id
                ).await?;
                println!("{}@{} ({})", user.name, user.domain, user.id);
            }

        },
        AccountCommands::Add(add_args) => {
            // TODO: switch for production
            // let api_url = util::get_api_url(&add_args.domain);
            let api_url = util::get_api_url("localhost:3000");
            let user = requests::fetch_user_by_id(
                client, &api_url, add_args.user_id
            ).await?;
            config.get_or_create_account_config(&user);
            save_config(config)?;
            println!(
                "Added account: {}@{} ({}).",
                user.name, user.domain, user.id
            );
        },
    }
    Ok(())
}
