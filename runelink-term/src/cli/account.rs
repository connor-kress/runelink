use runelink_client::{requests, util::get_api_url};
use runelink_types::SignupRequest;
use uuid::Uuid;

use crate::{
    cli::input::read_input, error::CliError, storage_auth::AccountAuth, util,
};

use super::{
    config::{DefaultAccountArgs, handle_default_account_commands},
    context::CliContext,
    input::unwrap_or_prompt,
    select::select_inline,
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
    /// Create a new account
    Create(NameAndDomainArgs),
    /// Login to an account (store authentication tokens)
    Login(LoginArgs),
    /// Logout from an account (remove authentication tokens)
    Logout(LogoutArgs),
    /// Show authentication status for an account
    Status(StatusArgs),
    /// Delete an account (deletes the underlying user)
    Delete(DeleteAccountArgs),
    /// Manage default account
    Default(DefaultAccountArgs),
}

#[derive(clap::Args, Debug)]
pub struct NameAndDomainArgs {
    /// The account's username
    #[clap(long)]
    pub name: Option<String>,
    /// The domain name of the account's host
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct LoginArgs {
    /// The account's username
    #[clap(long)]
    pub name: Option<String>,
    /// The domain name of the account's host
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct LogoutArgs {
    /// The account's username
    #[clap(long)]
    pub name: Option<String>,
    /// The domain name of the account's host
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct StatusArgs {
    /// The account's username
    #[clap(long)]
    pub name: Option<String>,
    /// The domain name of the account's host
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct DeleteAccountArgs {
    /// The ID of the user/account to delete.
    #[clap(long)]
    pub user_id: Option<Uuid>,
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
        }

        AccountCommands::Create(create_args) => {
            let domain =
                unwrap_or_prompt(create_args.domain.clone(), "Domain")?;
            let name = unwrap_or_prompt(create_args.name.clone(), "Name")?;
            let password = read_input("Password: ")?.ok_or_else(|| {
                CliError::InvalidArgument("Password is required.".into())
            })?;
            let api_url = get_api_url(&domain);
            let signup_req = SignupRequest { name, password };
            let user =
                requests::auth::signup(ctx.client, &api_url, &signup_req)
                    .await?;
            ctx.config.get_or_create_account_config(&user);
            ctx.config.save()?;
            println!("Created account: {}", user.verbose());
        }

        AccountCommands::Login(login_args) => {
            // Get or discover account
            let account = if let (Some(name), Some(domain)) =
                (&login_args.name, &login_args.domain)
            {
                // Try to find existing account in config
                if let Some(acc) =
                    ctx.config.get_account_config_by_name(name, domain)
                {
                    acc
                } else {
                    // Account doesn't exist, fetch it from server
                    let api_url = get_api_url(domain);
                    let user = requests::users::fetch_by_name_and_domain(
                        ctx.client,
                        &api_url,
                        name.clone(),
                        domain.clone(),
                    )
                    .await?;
                    ctx.config.get_or_create_account_config(&user);
                    ctx.config.save()?;
                    ctx.config
                        .get_account_config_by_name(name, domain)
                        .expect("Account should exist after creation")
                }
            } else {
                ctx.account.ok_or(CliError::MissingAccount)?
            };

            let password = read_input("Password: ")?.ok_or_else(|| {
                CliError::InvalidArgument("Password is required.".into())
            })?;

            // Generate a random client_id for this session
            let client_id = Uuid::new_v4().to_string();

            let api_url = get_api_url(&account.domain);
            let token_response = requests::auth::token_password(
                ctx.client,
                &api_url,
                &account.name,
                &password,
                None, // scope defaults to "openid" on server
                Some(&client_id),
            )
            .await?;

            // Store auth data
            let account_auth = AccountAuth {
                refresh_token: token_response.refresh_token,
                access_token: Some(token_response.access_token),
                expires_at: Some(
                    time::OffsetDateTime::now_utc().unix_timestamp()
                        + token_response.expires_in,
                ),
                client_id: Some(client_id),
                scope: None, // Server defaults to "openid"
            };
            ctx.auth_cache.set(account.user_id, account_auth);
            ctx.auth_cache.save()?;

            println!("Logged in successfully: {}", account.verbose());
        }

        AccountCommands::Logout(logout_args) => {
            let account = if let (Some(name), Some(domain)) =
                (&logout_args.name, &logout_args.domain)
            {
                ctx.config
                    .get_account_config_by_name(name, domain)
                    .ok_or_else(|| {
                        CliError::InvalidArgument("Account not found.".into())
                    })?
            } else {
                ctx.account.ok_or(CliError::MissingAccount)?
            };

            if ctx.auth_cache.remove(&account.user_id).is_some() {
                ctx.auth_cache.save()?;
                println!("Logged out successfully: {}", account.verbose());
            } else {
                println!(
                    "No authentication data found for: {}",
                    account.verbose()
                );
            }
        }

        AccountCommands::Status(status_args) => {
            let account = if let (Some(name), Some(domain)) =
                (&status_args.name, &status_args.domain)
            {
                ctx.config
                    .get_account_config_by_name(name, domain)
                    .ok_or_else(|| {
                        CliError::InvalidArgument("Account not found.".into())
                    })?
            } else {
                ctx.account.ok_or(CliError::MissingAccount)?
            };

            if let Some(auth) = ctx.auth_cache.get(&account.user_id) {
                println!("Account: {}", account.verbose());
                println!("  Authenticated: Yes");
                if let Some(expires_at) = auth.expires_at {
                    let expires =
                        time::OffsetDateTime::from_unix_timestamp(expires_at)
                            .unwrap_or_else(|_| {
                                time::OffsetDateTime::now_utc()
                            });
                    let now = time::OffsetDateTime::now_utc();
                    if expires > now {
                        let remaining = expires - now;
                        println!(
                            "  Access token expires in: {} seconds",
                            remaining.whole_seconds()
                        );
                    } else {
                        println!("  Access token: Expired");
                    }
                } else {
                    println!("  Access token: Not cached");
                }
                if let Some(ref client_id) = auth.client_id {
                    println!("  Client ID: {}", client_id);
                }
                if let Some(ref scope) = auth.scope {
                    println!("  Scope: {}", scope);
                }
            } else {
                println!("Account: {}", account.verbose());
                println!("  Authenticated: No");
            }
        }

        AccountCommands::Delete(delete_args) => {
            if ctx.config.accounts.is_empty() {
                return Err(CliError::InvalidArgument(
                    "No accounts in local config.".into(),
                ));
            }

            let (user_id, domain) = if let Some(user_id) = delete_args.user_id {
                let account = ctx
                    .config
                    .accounts
                    .iter()
                    .find(|a| a.user_id == user_id)
                    .ok_or_else(|| {
                        CliError::InvalidArgument(
                            "User ID not found in local config.".into(),
                        )
                    })?;
                (account.user_id, account.domain.clone())
            } else {
                let account = select_inline(
                    &ctx.config.accounts,
                    "Select account to delete",
                    crate::storage::AccountConfig::to_string,
                )?
                .ok_or(CliError::Cancellation)?;
                println!();
                (account.user_id, account.domain.clone())
            };

            // Derive API URL from the selected/matched account's domain
            let api_url = get_api_url(&domain);
            let access_token =
                ctx.get_access_token_for(user_id, &api_url).await?;

            // Delete the user on its home server
            requests::users::delete(
                ctx.client,
                &api_url,
                &access_token,
                user_id,
            )
            .await?;

            // Clean up local config/auth
            ctx.auth_cache.remove(&user_id);
            ctx.config.accounts.retain(|a| a.user_id != user_id);
            if ctx.config.default_account == Some(user_id) {
                ctx.config.default_account = None;
            }
            ctx.config.save()?;
            ctx.auth_cache.save()?;

            println!("Deleted account/user: {user_id}");
        }

        AccountCommands::Default(default_args) => {
            handle_default_account_commands(ctx, default_args).await?;
        }
    }
    Ok(())
}
