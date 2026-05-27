#[cfg(feature = "tui")]
pub mod ui;

#[cfg(feature = "tui")]
use crate::agent::event::AgentEvent;
#[cfg(feature = "tui")]
use crate::app::{App, AppMode, ChatMessage, ToolCallInfo};
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

    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(256);

    let version = app.version.clone();
    app.messages.push(ChatMessage {
        role: "assistant".into(),
        content: format!(
            "Welcome to i-rs-code v{version}\n\n\
             Type a message to start coding...\n\n\
             可用命令:\n  \
             i-rs-code chat <prompt>  一次性对话\n  \
             i-rs-code config init    交互式配置\n  \
             i-rs-code config show    查看配置"
        ),
    });

    while !app.should_quit {
        terminal.draw(|f| {
            ui::render(f, &app);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_key(key, &mut app, &event_tx).await;
            }
        }

        if let Ok(event) = event_rx.try_recv() {
            handle_event(event, &mut app);
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
fn handle_event(event: AgentEvent, app: &mut App) {
    match event {
        AgentEvent::Token(t) => {
            app.push_token(&t);
        }
        AgentEvent::ToolCallStart { id: _id, name, args } => {
            let info = ToolCallInfo {
                name,
                args: serde_json::to_string_pretty(&args).unwrap_or_default(),
                result: None,
            };
            if let Some(ref mut s) = app.streaming {
                s.current_tool = Some(info);
            }
        }
        AgentEvent::ToolCallEnd { id: _id, name: _name, result } => {
            if let Some(ref mut s) = app.streaming {
                if let Some(mut tool) = s.current_tool.take() {
                    tool.result = Some(result);
                    s.tool_calls.push(tool);
                }
            }
        }
        AgentEvent::Done { usage } => {
            let content = app.finish_streaming();
            if let Some(u) = usage {
                app.add_token_usage(u.input_tokens, u.output_tokens);
            }
            app.messages.push(ChatMessage {
                role: "assistant".into(),
                content,
            });
            app.scroll_offset = 0;
            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
        AgentEvent::Error(e) => {
            let content = app.finish_streaming();
            let msg = if !content.is_empty() {
                format!("{}\n\nError: {}", content, e)
            } else {
                format!("Error: {}", e)
            };
            app.messages.push(ChatMessage {
                role: "assistant".into(),
                content: msg,
            });
            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
    }
}

#[cfg(feature = "tui")]
async fn handle_key(key: KeyEvent, app: &mut App, event_tx: &mpsc::Sender<AgentEvent>) {
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
        KeyCode::Enter if key.modifiers == KeyModifiers::ALT && matches!(app.mode, AppMode::Idle) => {
            app.insert_char('\n');
        }
        KeyCode::Enter if matches!(app.mode, AppMode::Idle) && !app.input.is_empty() => {
            let prompt = std::mem::take(&mut app.input);
            app.cursor_pos = 0;
            app.messages.push(ChatMessage {
                role: "user".into(),
                content: prompt.clone(),
            });
            app.start_streaming();
            app.mode = AppMode::Waiting;

            let config = app.config.clone();
            let tx = event_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = run_streaming_agent(&config, &prompt, tx.clone()).await {
                    tx.send(AgentEvent::Error(e.to_string())).await.ok();
                }
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
async fn run_streaming_agent(
    config: &crate::config::Config,
    prompt: &str,
    event_tx: mpsc::Sender<AgentEvent>,
) -> anyhow::Result<()> {
    let provider = crate::provider::create_provider(config)?;
    let tools = crate::tools::ToolRegistry::new(config)?;
    let mut agent = crate::agent::Agent::new(config.clone(), provider, tools, false);
    agent.run_once_streaming(prompt, event_tx).await?;
    Ok(())
}

#[cfg(not(feature = "tui"))]
pub async fn run(_app: crate::app::App) -> anyhow::Result<()> {
    anyhow::bail!("TUI feature not enabled. Build with --features tui")
}
