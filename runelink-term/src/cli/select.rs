use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use runelink_client::{requests, util::get_api_url};
use runelink_types::{Channel, Server};
use std::{collections::HashSet, io::Write};
use uuid::Uuid;

use crate::error::CliError;

use super::context::CliContext;

pub fn select_inline<'a, T, F>(
    items: &'a [T],
    prompt: &str,
    display: F,
) -> std::io::Result<Option<&'a T>>
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        println!("(no items to select)");
        return Ok(None);
    }
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(
        stdout,
        Hide,
        MoveToColumn(0),
        Print(format!("{}\n", prompt))
    )?;

    for (i, item) in items.iter().enumerate() {
        let prefix = if i == 0 { "> " } else { "  " };
        execute!(
            stdout,
            MoveToColumn(0),
            Print(format!("{}{}\n", prefix, display(item)))
        )?;
    }
    stdout.flush()?;

    let mut selected = 0;
    loop {
        if let Event::Key(KeyEvent {
            kind: KeyEventKind::Press,
            code,
            modifiers,
            ..
        }) = crossterm::event::read()?
        {
            match (code, modifiers) {
                // Ctrl-C - propagate as Interrupted
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    execute!(stdout, MoveToColumn(0))?;
                    disable_raw_mode()?;
                    execute!(stdout, Show)?;
                    panic!("Interrupted by Ctrl-C");
                }
                // Esc or 'q' - cancel
                (KeyCode::Esc, _) | (KeyCode::Char('q'), _) => {
                    execute!(stdout, MoveToColumn(0))?;
                    disable_raw_mode()?;
                    execute!(stdout, Show)?;
                    return Ok(None);
                }
                // Enter - confirm
                (KeyCode::Enter, _) => {
                    execute!(
                        stdout,
                        MoveUp(items.len() as u16),
                        MoveToColumn(0),
                        Clear(ClearType::FromCursorDown),
                        Print(format!("> {}\n", display(&items[selected]))),
                        MoveToColumn(0),
                    )?;
                    stdout.flush()?;
                    disable_raw_mode()?;
                    execute!(stdout, Show)?;
                    return Ok(Some(&items[selected]));
                }
                // Up or 'k'
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    selected = if selected == 0 {
                        items.len() - 1
                    } else {
                        selected - 1
                    };
                }
                // Down or 'j'
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    selected = (selected + 1) % items.len();
                }
                _ => {}
            }

            // Redraw the list in place
            execute!(stdout, MoveUp(items.len() as u16))?;
            for (i, item) in items.iter().enumerate() {
                let prefix = if i == selected { "> " } else { "  " };
                execute!(
                    stdout,
                    Clear(ClearType::CurrentLine),
                    MoveToColumn(0),
                    Print(format!("{}{}\n", prefix, display(item)))
                )?;
            }
            stdout.flush()?;
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ServerSelectionType<'a> {
    MemberOnly,
    All { domain: &'a str },
    NonMemberOnly { domain: &'a str },
}

pub async fn get_server_selection(
    ctx: &CliContext<'_>,
    selection_type: ServerSelectionType<'_>,
) -> Result<Server, CliError> {
    let servers = match selection_type {
        ServerSelectionType::All { domain } => {
            let api_url = get_api_url(domain);
            requests::servers::fetch_all(ctx.client, &api_url).await?
        }
        ServerSelectionType::MemberOnly => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let account_api_url = get_api_url(&account.domain);
            requests::servers::fetch_by_user(
                ctx.client,
                &account_api_url,
                account.user_id,
            )
            .await?
        }
        ServerSelectionType::NonMemberOnly { domain } => {
            let account = ctx.account.ok_or(CliError::MissingAccount)?;
            let account_api_url = get_api_url(&account.domain);
            let api_url = get_api_url(domain);
            let all_servers =
                requests::servers::fetch_all(ctx.client, &api_url).await?;
            let member_servers = requests::servers::fetch_by_user(
                ctx.client,
                &account_api_url,
                account.user_id,
            )
            .await?;
            let member_server_ids = member_servers
                .iter()
                .map(|s| s.id)
                .collect::<HashSet<Uuid>>();
            all_servers
                .into_iter()
                .filter(|s| !member_server_ids.contains(&s.id))
                .collect()
        }
    };
    if servers.is_empty() {
        return Err(CliError::NoActionPossible(format!(
            "No applicable servers (viewing {:?}).\n\
                    For more information, try `rune server --help`.",
            selection_type,
        )));
    }
    let server = select_inline(&servers, "Select server", Server::to_string)?
        .ok_or(CliError::Cancellation)?;
    println!();
    Ok(server.clone())
}

pub async fn get_channel_selection(
    ctx: &CliContext<'_>,
    api_url: &str,
    server_id: Uuid,
) -> Result<Channel, CliError> {
    let channels =
        requests::channels::fetch_by_server(ctx.client, api_url, server_id)
            .await?;
    if channels.is_empty() {
        return Err(CliError::NoActionPossible(
            "No channels available.\n\
                For more information, try `rune channel --help`."
                .into(),
        ));
    }
    let channel =
        select_inline(&channels, "Select channel", Channel::to_string)?
            .ok_or(CliError::Cancellation)?;
    println!();
    Ok(channel.clone())
}

pub async fn get_channel_selection_with_inputs(
    ctx: &CliContext<'_>,
    channel_id: Option<Uuid>,
    server_id: Option<Uuid>,
) -> Result<(Server, Channel), CliError> {
    match (channel_id, server_id) {
        (Some(channel_id), Some(server_id)) => {
            let api_url = ctx.config.try_get_server_api_url(server_id)?;
            let server =
                requests::servers::fetch_by_id(ctx.client, &api_url, server_id)
                    .await?;
            let channel = requests::channels::fetch_by_id(
                ctx.client, &api_url, server_id, channel_id,
            )
            .await?;
            Ok((server, channel))
        }
        (Some(_channel_id), None) => Err(CliError::MissingContext(
            "Server ID must be passed with channel ID.".into(),
        )),
        (None, Some(server_id)) => {
            let api_url = ctx.config.try_get_server_api_url(server_id)?;
            let server =
                requests::servers::fetch_by_id(ctx.client, &api_url, server_id)
                    .await?;
            let channel =
                get_channel_selection(ctx, &api_url, server.id).await?;
            Ok((server, channel))
        }
        (None, None) => {
            let server =
                get_server_selection(ctx, ServerSelectionType::MemberOnly)
                    .await?;
            let api_url = get_api_url(&server.domain);
            let channel =
                get_channel_selection(ctx, &api_url, server.id).await?;
            Ok((server, channel))
        }
    }
}
