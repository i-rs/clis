#[cfg(feature = "tui")]
pub mod ui;

#[cfg(feature = "tui")]
use crate::app::App;
#[cfg(feature = "tui")]
use std::io;

#[cfg(feature = "tui")]
pub async fn run(app: App) -> anyhow::Result<()> {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| {
            ui::render(f, &app);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => should_quit = true,
                KeyCode::Esc => should_quit = true,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(not(feature = "tui"))]
pub async fn run(_app: crate::app::App) -> anyhow::Result<()> {
    anyhow::bail!("TUI feature not enabled. Build with --features tui")
}
