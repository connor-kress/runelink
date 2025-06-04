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

pub async fn get_channel_selection(
    ctx: &CliContext<'_>,
    channel_id: Option<Uuid>,
    server_id: Option<Uuid>,
) -> Result<Channel, CliError> {
    match (channel_id, server_id) {
        (Some(channel_id), Some(server_id)) => {
            let server_api_url = ctx.config.try_get_server_api_url(server_id)?;
            requests::fetch_channel_by_id(
                ctx.client,
                &server_api_url,
                channel_id,
            ).await
        },
        (Some(_channel_id), None) => {
            Err(CliError::MissingContext(
                "Server ID must be passed with channel ID.".into(),
            ))
        }
        (None, Some(server_id)) => {
            // ctx.config.get_default_channel(server_id)
            let server_api_url = ctx.config.try_get_server_api_url(server_id)?;
            let channels = requests::fetch_channels_by_server(
                ctx.client, &server_api_url, server_id
            ).await?;
            let channel = select_inline(
                &channels,
                "Select channel",
                Channel::to_string,
            )?
            .ok_or(CliError::Cancellation)?;
            println!();
            Ok(channel.clone())
        },
        (None, None) => {
            // ctx.config.default_server.and_then(|server_id| {
            //     ctx.config.get_default_channel(server_id)
            // })
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
            let server_api_url = get_api_url(&server.domain);
            let channels = requests::fetch_channels_by_server(
                ctx.client, &server_api_url, server.id
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
        },
    }
}
