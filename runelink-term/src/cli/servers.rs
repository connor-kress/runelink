use runelink_client::{requests, util::get_api_url};
use runelink_types::{NewServer, NewServerMember, ServerRole};
use uuid::Uuid;

use crate::{error::CliError, util::group_memberships_by_host};

use super::{
    config::{handle_default_server_commands, DefaultServerArgs},
    context::CliContext,
    domain_query::DomainQueryBuilder,
    input::read_input,
    select::{get_server_selection, ServerSelectionType},
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
            let api_url = get_api_url(&account.domain);
            let memberships = requests::fetch_server_memberships_by_user(
                ctx.client,
                &api_url,
                account.user_id,
            ).await?;
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
                println!("{}", domain);
                for membership in memberships {
                    let server = &membership.server;
                    print!("    {} ({})", server.title, server.id);
                    if membership.role == ServerRole::Admin {
                        println!(" - admin");
                    } else {
                        println!();
                    }
                }
            }
        }

        ServerCommands::Get(get_args) => {
            let api_url = DomainQueryBuilder::new(ctx)
                .try_domain(get_args.domain.clone())
                .try_server(Some(get_args.server_id))
                .get_api_url()?;
            let server = requests::fetch_server_by_id(
                ctx.client, &api_url, get_args.server_id
            ).await?;
            println!("{} / {} ({})", server.domain, server.title, server.id);
        }

        ServerCommands::Create(create_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let api_url = DomainQueryBuilder::new(ctx)
                .try_domain(create_args.domain.clone())
                .get_api_url()?;
            // TODO: servers can't handle cross host server membership yet.
            // We need to sync the membership with another request to the home
            // server too.
            let title = if let Some(title) = &create_args.title {
                title.clone()
            } else {
                read_input("Server Title: ")?
                    .ok_or_else(|| CliError::InvalidArgument(
                        "Server title is required.".into()
                    ))?
            };
            let desc = if create_args.description.is_some() {
                create_args.description.clone()
            } else if create_args.no_description {
                None
            } else {
                read_input("Server Description (leave blank for none):\n> ")?
            };
            let new_server = NewServer {
                title,
                description: desc,
                user_domain: account.domain.clone(),
                user_id: account.user_id,
            };
            let server = requests::create_server(
                ctx.client, &api_url, &new_server
            ).await?;
            ctx.config.get_or_create_server_config(&server, &account.domain);
            ctx.config.save()?;
            println!(
                "Created server: {} ({}).",
                server.title, server.id
            );
        }

        ServerCommands::Join(join_args) => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let (domain, api_url) = DomainQueryBuilder::new(ctx)
                .try_domain(join_args.domain.clone())
                .try_server(join_args.server_id)
                .get_domain_and_api_url()?;
            let server = if let Some(server_id) = join_args.server_id {
                requests::fetch_server_by_id(
                    ctx.client, &api_url, server_id
                ).await?
            } else {
                get_server_selection(
                    ctx, ServerSelectionType::NonMemberOnly { domain: &domain }
                ).await?
            };
            let new_member = NewServerMember::member(
                account.user_id,
                account.domain.clone(),
            );
            let _member = requests::join_server(
                ctx.client, &api_url, server.id, &new_member
            ).await?;
            ctx.config.get_or_create_server_config(&server, &account.domain);
            ctx.config.save()?;
            println!("Joined server: {} ({}).", server.title, server.id);
        }

        ServerCommands::Default(default_args) => {
            handle_default_server_commands(ctx, &default_args).await?;
        }
    }
    Ok(())
}
