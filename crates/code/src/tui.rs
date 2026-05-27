#[cfg(feature = "tui")]
pub mod ui;

#[cfg(feature = "tui")]
use crate::app::{App, AppMode, ChatMessage};
#[cfg(feature = "tui")]
use std::io;
#[cfg(feature = "tui")]
use std::time::Duration;
#[cfg(feature = "tui")]
use tokio::sync::mpsc;
#[cfg(feature = "tui")]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(feature = "tui")]
use ratatui::{backend::CrosstermBackend, Terminal};

#[cfg(feature = "tui")]
pub async fn run(mut app: App) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (result_tx, mut result_rx) = mpsc::channel::<String>(4);

    while !app.should_quit {
        terminal.draw(|f| {
            ui::render(f, &app);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_key(key, &mut app, &result_tx).await;
            }
        }

        if let Ok(response) = result_rx.try_recv() {
            app.messages.push(ChatMessage {
                role: "assistant".into(),
                content: response,
            });
            app.scroll_offset = 0;
            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
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

#[cfg(feature = "tui")]
async fn handle_key(key: KeyEvent, app: &mut App, result_tx: &mpsc::Sender<String>) {
    match key.code {
        KeyCode::Char('q') if matches!(app.mode, AppMode::Idle) => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            if matches!(app.mode, AppMode::Idle) {
                app.should_quit = true;
            }
        }
        KeyCode::Char(c) if matches!(app.mode, AppMode::Idle) => {
            if key.modifiers == KeyModifiers::CONTROL && c == 'c' {
                app.should_quit = true;
                return;
            }
            app.insert_char(c);
        }
        KeyCode::Backspace if matches!(app.mode, AppMode::Idle) => {
            app.delete_char();
        }
        KeyCode::Delete if matches!(app.mode, AppMode::Idle) => {
            if app.cursor_pos < app.input.len() {
                app.input.remove(app.cursor_pos);
            }
        }
        KeyCode::Left if matches!(app.mode, AppMode::Idle) => {
            app.move_cursor_left();
        }
        KeyCode::Right if matches!(app.mode, AppMode::Idle) => {
            app.move_cursor_right();
        }
        KeyCode::Home if matches!(app.mode, AppMode::Idle) => {
            app.move_cursor_home();
        }
        KeyCode::End if matches!(app.mode, AppMode::Idle) => {
            app.move_cursor_end();
        }
        KeyCode::Up if matches!(app.mode, AppMode::Idle) => {
            app.scroll_up();
        }
        KeyCode::Down if matches!(app.mode, AppMode::Idle) => {
            app.scroll_down();
        }
        KeyCode::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_add(10);
        }
        KeyCode::Enter if matches!(app.mode, AppMode::Idle) && !app.input.is_empty() => {
            let prompt = std::mem::take(&mut app.input);
            app.cursor_pos = 0;
            app.messages.push(ChatMessage {
                role: "user".into(),
                content: prompt.clone(),
            });
            app.mode = AppMode::Waiting;

            let config = app.config.clone();
            let tx = result_tx.clone();
            tokio::spawn(async move {
                let result = run_agent(&config, &prompt).await;
                tx.send(result.unwrap_or_else(|e| format!("Error: {}", e)))
                    .await
                    .ok();
            });
        }
        KeyCode::Tab if matches!(app.mode, AppMode::Idle) => {
            app.insert_char(' ');
            app.insert_char(' ');
        }
        _ => {}
    }
}

#[cfg(feature = "tui")]
async fn run_agent(config: &crate::config::Config, prompt: &str) -> anyhow::Result<String> {
    use crate::agent::Agent;
    use crate::provider::{self, LlmMessage};
    use crate::tools::ToolRegistry;

    let provider = provider::create_provider(config)?;
    let tools = ToolRegistry::new(config)?;
    let mut agent = Agent::new(config.clone(), provider, tools, false);

    let system_text = "You are i-rs-code, a code editor AI agent running in TUI mode. \
        You can read/write files, execute commands, create i-rs CLI tools, and more. \
        Always use the available tools to help the user. \
        After making changes, verify with cargo check or equivalent commands.";

    let tool_defs = agent.tools.schemas();

    let mut msgs = Vec::new();
    msgs.push(LlmMessage::System(system_text.to_string()));
    for msg in &agent.messages {
        match msg {
            LlmMessage::User(c) => msgs.push(LlmMessage::User(c.clone())),
            LlmMessage::Assistant(c) => msgs.push(LlmMessage::Assistant(c.clone())),
            LlmMessage::ToolCall { id, name, args } => {
                msgs.push(LlmMessage::ToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    args: args.clone(),
                });
            }
            LlmMessage::Tool {
                name,
                content,
                call_id,
            } => {
                msgs.push(LlmMessage::Tool {
                    name: name.clone(),
                    content: content.clone(),
                    call_id: call_id.clone(),
                });
            }
            _ => {}
        }
    }
    msgs.push(LlmMessage::User(prompt.to_string()));

    let (final_text, new_messages) =
        crate::agent::engine::react_loop(&*agent.provider, &agent.tools, msgs, &tool_defs, false)
            .await?;

    agent.messages = new_messages;

    Ok(final_text)
}

#[cfg(not(feature = "tui"))]
pub async fn run(_app: crate::app::App) -> anyhow::Result<()> {
    anyhow::bail!("TUI feature not enabled. Build with --features tui")
}
