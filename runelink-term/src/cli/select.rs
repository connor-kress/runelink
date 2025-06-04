use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use runelink_types::{Channel, Server};
use std::io::Write;
use uuid::Uuid;

use crate::{error::CliError, requests, util::get_api_url};

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
    execute!(stdout, Hide)?;
    execute!(stdout, MoveToColumn(0), Print(format!("{}\n", prompt)))?;

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
            code, modifiers, ..
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
                    execute!(stdout, MoveToColumn(0))?;
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

pub async fn get_server_selection(
    ctx: &CliContext<'_>,
) -> Result<Server, CliError> {
        let account = ctx.account.ok_or(CliError::MissingAccount)?;
        let account_api_url = get_api_url(&account.domain);
        let memberships = requests::fetch_server_memberships_by_user(
            ctx.client,
            &account_api_url,
            account.user_id,
        ).await?;
        if memberships.is_empty() {
            return Err(CliError::NoActionPossible(
                "No servers joined. See `rune servers --help`.".into()
            ));
        }
        let servers = memberships
            .into_iter()
            .map(|m| m.server)
            .collect::<Vec<Server>>();
        let server = select_inline(
            &servers,
            "Select server",
            Server::to_string,
        )?
        .ok_or(CliError::Cancellation)?;
        println!();
        Ok(server.clone())
}

pub async fn get_channel_selection(
    ctx: &CliContext<'_>,
    api_url: &str,
    server_id: Uuid,
) -> Result<Channel, CliError> {
        let channels = requests::fetch_channels_by_server(
            ctx.client, api_url, server_id
        ).await?;
        if channels.is_empty() {
            return Err(CliError::NoActionPossible(
                "No channels available. See `rune channels create --help`."
                    .into()
            ));
        }
        let channel = select_inline(
            &channels,
            "Select channel",
            Channel::to_string,
        )?
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
            let server = requests::fetch_server_by_id(
                ctx.client,
                &api_url,
                server_id,
            ).await?;
            let channel = requests::fetch_channel_by_id(
                ctx.client,
                &api_url,
                channel_id,
            ).await?;
            Ok((server, channel))
        },
        (Some(_channel_id), None) => {
            Err(CliError::MissingContext(
                "Server ID must be passed with channel ID.".into(),
            ))
        }
        (None, Some(server_id)) => {
            let api_url = ctx.config.try_get_server_api_url(server_id)?;
            let server = requests::fetch_server_by_id(
                ctx.client,
                &api_url,
                server_id,
            ).await?;
            let channel =
                get_channel_selection(ctx, &api_url, server.id).await?;
            Ok((server, channel))
        },
        (None, None) => {
            let server = get_server_selection(ctx).await?;
            let api_url = get_api_url(&server.domain);
            let channel =
                get_channel_selection(ctx, &api_url, server.id).await?;
            Ok((server, channel))
        },
    }
}
