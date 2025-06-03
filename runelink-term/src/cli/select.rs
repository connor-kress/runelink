use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use runelink_types::Server;
use std::io::Write;
use time::OffsetDateTime;
use uuid::Uuid;

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

pub fn demo_select_inline() -> std::io::Result<()> {
    let servers = vec![
        Server {
            id: Uuid::new_v4(),
            title: "Darplex".into(),
            description: Some("We play games".into()),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        },
        Server {
            id: Uuid::new_v4(),
            title: "Other server".into(),
            description: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        },
        Server {
            id: Uuid::new_v4(),
            title: "Mandar fanclub".into(),
            description: Some("Very large".into()),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        },
    ];

    let show = |s: &Server| {
        if let Some(desc) = &s.description {
            format!("{} - {}", s.title, desc)
        } else {
            s.title.clone()
        }
    };

    if let Some(sel) = select_inline(&servers, "Select primary server:", show)?
    {
        println!("You picked: {} ({})", sel.title, sel.id);
    } else {
        println!("Primary selection cancelled.");
    }
    println!();

    if let Some(sel) = select_inline(&servers, "Select another server:", show)?
    {
        println!("You picked: {} ({})", sel.title, sel.id);
    } else {
        println!("Other selection cancelled.");
    }
    Ok(())
}
