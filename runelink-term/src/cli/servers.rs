#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use runelink_client::requests;
use runelink_types::{NewServer, NewServerMembership, ServerRole};
use uuid::Uuid;

use crate::{error::CliError, util::group_memberships_by_host};

use super::{
    config::{DefaultServerArgs, handle_default_server_commands},
    context::CliContext,
    input::{read_input, unwrap_or_prompt},
};

#[derive(clap::Args, Debug)]
pub struct ServerArgs {
    #[clap(subcommand)]
    pub command: ServerCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum ServerCommands {
    /// List all servers
    List,
    /// Get a server by ID
    Get(ServerGetArg),
    /// Create a new server
    Create(ServerCreateArgs),
    /// Create a new server
    Join(ServerJoinArgs),
    /// Manage default server
    Default(DefaultServerArgs),
}

#[derive(clap::Args, Debug)]
pub struct ServerGetArg {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Uuid,
    /// The domain of the server
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ServerCreateArgs {
    /// The title of the server
    #[clap(long)]
    pub title: Option<String>,
    /// The description of the server
    #[clap(long)]
    pub description: Option<String>,
    /// Skip description cli prompt
    #[clap(long)]
    pub no_description: bool,
    /// The domain of the server
    #[clap(long)]
    pub domain: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ServerJoinArgs {
    /// The ID of the server
    #[clap(long)]
    pub server_id: Option<Uuid>,
    /// The domain of the server
    #[clap(long)]
    pub domain: Option<String>,
}

pub async fn handle_server_commands(
    ctx: &mut CliContext<'_>,
    server_args: &ServerArgs,
) -> Result<(), CliError> {
    match &server_args.command {
        ServerCommands::List => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;
            let memberships = requests::memberships::fetch_by_user(
                ctx.client,
                &api_url,
                account.user_id,
            )
            .await?;
            if memberships.is_empty() {
                println!(
                    "No servers joined.\n\
                    For more information, try `rune server --help`."
                )
            }
            let mut is_first = true;
            for (domain, memberships) in group_memberships_by_host(&memberships)
            {
                if is_first {
                    is_first = false
                } else {
                    println!(); // separation between host groups
                }
                println!("{domain}");
                for membership in memberships {
                    let server = &membership.server;
                    print!("    {}", server.verbose());
                    if membership.role == ServerRole::Admin {
                        println!(" - admin");
                    } else {
                        println!();
                    }
                }
            }
        }

        ServerCommands::Get(get_args) => {
            let api_url = ctx.home_api_url()?;
            let server = requests::servers::fetch_by_id(
                ctx.client,
                &api_url,
                get_args.server_id,
                get_args.domain.as_deref(),
            )
            .await?;
            println!(
                "{domain} / {title} ({id})",
                domain = server.domain,
                title = server.title,
                id = server.id
            );
        }

        ServerCommands::Create(create_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            let title =
                unwrap_or_prompt(create_args.title.clone(), "Server Title")?;
            let description = if create_args.description.is_some() {
                create_args.description.clone()
            } else if create_args.no_description {
                None
            } else {
                read_input("Server Description (leave blank for none):\n> ")?
            };
            let new_server = NewServer { title, description };
            let server = requests::servers::create(
                ctx.client,
                &api_url,
                &access_token,
                &new_server,
                create_args.domain.as_deref(),
            )
            .await?;
            ctx.config
                .get_or_create_server_config(&server, &account.domain);
            ctx.config.save()?;
            println!("Created server: {}", server.verbose());
        }

        ServerCommands::Join(join_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = ctx.home_api_url()?;
            let access_token = ctx.get_access_token().await?;
            // For now, we'll need to fetch server info from home server
            // TODO: Add proxy discovery endpoints for remote servers
            let server = if let Some(server_id) = join_args.server_id {
                requests::servers::fetch_by_id(
                    ctx.client,
                    &api_url,
                    server_id,
                    join_args.domain.as_deref(),
                )
                .await?
            } else {
                // For now, require server_id if domain not provided
                // TODO: Implement server selection via home server proxy
                return Err(CliError::InvalidArgument(
                    "Server ID required. Remote server discovery not yet implemented.".into(),
                ));
            };
            let new_member = NewServerMembership {
                user_id: account.user_id,
                user_domain: account.domain.clone(),
                server_id: server.id,
                server_domain: server.domain.clone(),
                role: ServerRole::Member,
            };
            let _member = requests::memberships::create(
                ctx.client,
                &api_url,
                &access_token,
                server.id,
                &new_member,
            )
            .await?;
            ctx.config
                .get_or_create_server_config(&server, &account.domain);
            ctx.config.save()?;
            println!("Joined server: {}", server.verbose());
        }

        ServerCommands::Default(default_args) => {
            handle_default_server_commands(ctx, default_args).await?;
        }
    }
    Ok(())
}
