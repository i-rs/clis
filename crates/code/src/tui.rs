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
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind},
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

    if app.messages.is_empty() {
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
            reasoning: String::new(),
            tool_calls: None,
        });
    }

    while !app.should_quit {
        terminal.draw(|f| {
            ui::render(f, &app);
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key(key, &mut app, &event_tx).await;
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            app.scroll_offset = app.scroll_offset.saturating_add(3);
                        }
                        MouseEventKind::ScrollDown => {
                            app.scroll_offset = app.scroll_offset.saturating_sub(3);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if let Ok(event) = event_rx.try_recv() {
            handle_event(event, &mut app);
        }
    }

    // Auto-save session before exit
    let stats = if app.messages.len() > 1 {
        let tool_count: usize = app.messages.iter().filter(|m| m.role == "tool").count();
        let session_id = app.session_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let sessions_dir = crate::config::i_rs_code_dir().join("sessions");
        let session = crate::session::Session::from_chat_messages(Some(session_id.clone()), &app.messages);
        if session.save(&sessions_dir).is_ok() {
            Some((session_id, app.messages.len(), tool_count))
        } else {
            None
        }
    } else {
        None
    };

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    println!();
    println!("  ✨ 已退出 i-rs-code");
    if let Some((sid, msg_count, tool_count)) = stats {
        println!("  📊 {} 条消息 · {} 次工具调用", msg_count - 1, tool_count);
        println!("  ↻ 重新进入: i-rs-code tui --session {}", sid);
    }
    println!();

    Ok(())
}

#[cfg(feature = "tui")]
fn handle_event(event: AgentEvent, app: &mut App) {
    match event {
        AgentEvent::Token(t) => {
            app.push_token(&t);
        }
        AgentEvent::Reasoning(r) => {
            if let Some(ref mut s) = app.streaming {
                s.reasoning.push_str(&r);
            }
        }
        AgentEvent::ToolCallStart { id: _id, name, args } => {
            if matches!(name.as_str(), "write" | "edit")
                && let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                    app.file_changes.insert(path.to_string());
                    if let Ok(content) = std::fs::read_to_string(path) {
                        app.last_file_states.push((path.to_string(), content));
                    }
                }
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
            if let Some(ref mut s) = app.streaming
                && let Some(mut tool) = s.current_tool.take()
            {
                tool.result = Some(result);
                s.tool_calls.push(tool);
            }
        }
        AgentEvent::Status(_msg) => {
            // Status updates for provider retry etc. Could display in TUI status bar.
        }
        AgentEvent::FileChanged { path } => {
            app.file_changes.insert(path);
        }
        AgentEvent::Done { usage, messages, context_pct } => {
            let (content, reasoning) = app.finish_streaming();
            if !messages.is_empty() {
                app.agent_messages = messages.clone();
            }
            if let Some(u) = usage {
                app.add_token_usage(u.input_tokens, u.output_tokens);
            }
            app.context_usage = Some(context_pct);
            let tool_calls = extract_tool_calls(&messages);
            app.messages.push(ChatMessage {
                role: "assistant".into(),
                content,
                reasoning,
                tool_calls,
            });
            app.scroll_offset = 0;
            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
        AgentEvent::Error(e) => {
            let (content, reasoning) = app.finish_streaming();
            let msg = if !content.is_empty() {
                format!("{}\n\nError: {}", content, e)
            } else {
                format!("Error: {}", e)
            };
            app.messages.push(ChatMessage {
                role: "assistant".into(),
                content: msg,
                reasoning,
                tool_calls: None,
            });
            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
    }
}

#[cfg(feature = "tui")]
async fn handle_key(key: KeyEvent, app: &mut App, event_tx: &mpsc::Sender<AgentEvent>) {
    if app.show_shortcuts {
        app.show_shortcuts = false;
        return;
    }

    match app.mode {
        AppMode::Waiting => {
            match key.code {
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    if let Some(tx) = app.cancel_tx.take() {
                        tx.send(()).ok();
                    }
                    if let Some(h) = app.task_handle.take() {
                        h.abort();
                    }
                    app.mode = AppMode::Idle;
                    let (content, reasoning) = app.finish_streaming();
                    if !content.is_empty() {
                        app.messages.push(ChatMessage {
                            role: "assistant".into(),
                            content: format!("{}\n\n[Cancelled]", content),
                            reasoning,
                            tool_calls: None,
                        });
                    }
                    return;
                }
                KeyCode::Up => app.scroll_up(),
                KeyCode::Down => app.scroll_down(),
                KeyCode::PageUp => app.scroll_offset = app.scroll_offset.saturating_add(10),
                KeyCode::PageDown => app.scroll_offset = app.scroll_offset.saturating_sub(10),
                _ => {}
            }
            return;
        }
        AppMode::Idle => {}
    }

    match key.code {
        KeyCode::Char('z') if key.modifiers == KeyModifiers::CONTROL => {
            if let Some((path, content)) = app.last_file_states.pop() {
                match std::fs::write(&path, &content) {
                    Ok(_) => {
                        app.messages.push(ChatMessage {
                            role: "system".into(),
                            content: format!("Reverted {}", path),
                            reasoning: String::new(),
                            tool_calls: None,
                        });
                    }
                    Err(e) => {
                        app.messages.push(ChatMessage {
                            role: "system".into(),
                            content: format!("Failed to revert {}: {}", path, e),
                            reasoning: String::new(),
                            tool_calls: None,
                        });
                    }
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            if app.show_shortcuts {
                app.show_shortcuts = false;
            } else if app.show_debug {
                app.show_debug = false;
                app.debug_scroll = 0;
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('?') if !app.show_debug => {
            app.show_shortcuts = !app.show_shortcuts;
        }
        KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
            app.show_shortcuts = false;
            app.show_debug = !app.show_debug;
            if !app.show_debug {
                app.debug_scroll = 0;
            }
        }
        KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL && app.show_debug => {
            crate::debug::clear_log();
        }
        KeyCode::Up if app.show_debug => {
            app.debug_scroll = app.debug_scroll.saturating_sub(1);
        }
        KeyCode::Down if app.show_debug => {
            app.debug_scroll = app.debug_scroll.saturating_add(1);
        }
        KeyCode::PageUp if app.show_debug => {
            app.debug_scroll = app.debug_scroll.saturating_sub(10);
        }
        KeyCode::PageDown if app.show_debug => {
            app.debug_scroll = app.debug_scroll.saturating_add(10);
        }
        KeyCode::Char(c) => {
            if key.modifiers == KeyModifiers::CONTROL && c == 'c' {
                app.should_quit = true;
                return;
            }
            app.insert_char(c);
        }
        KeyCode::Backspace => app.delete_char(),
        KeyCode::Delete if app.cursor_pos < app.input.len() => {
            let len = app.input[app.cursor_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            app.input.drain(app.cursor_pos..app.cursor_pos + len);
        }
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => app.move_cursor_right(),
        KeyCode::Home => app.move_cursor_home(),
        KeyCode::End => app.move_cursor_end(),
        KeyCode::Up => app.scroll_up(),
        KeyCode::Down => app.scroll_down(),
        KeyCode::PageUp => app.scroll_offset = app.scroll_offset.saturating_add(10),
        KeyCode::PageDown => app.scroll_offset = app.scroll_offset.saturating_sub(10),
        KeyCode::Enter if key.modifiers == KeyModifiers::ALT => app.insert_char('\n'),
        KeyCode::Enter if !app.input.is_empty() => {
            let prompt = std::mem::take(&mut app.input);
            let expanded = expand_file_refs(&prompt);
            app.cursor_pos = 0;
            app.messages.push(ChatMessage {
                role: "user".into(),
                content: prompt.clone(),
                reasoning: String::new(),
                tool_calls: None,
            });
            app.start_streaming();
            app.mode = AppMode::Waiting;

            let config = app.config.clone();
            let tx = event_tx.clone();
            let history = std::mem::take(&mut app.agent_messages);
            let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
            app.cancel_tx = Some(cancel_tx);
            app.task_handle = Some(tokio::spawn(async move {
                if let Err(e) = run_streaming_agent(&config, &expanded, tx.clone(), history, cancel_rx).await {
                    tx.send(AgentEvent::Error(e.to_string())).await.ok();
                }
            }));
        }
        KeyCode::Tab => {
            let input = &app.input;
            let trimmed = input.trim();
            let matches: Vec<&str> = app.tool_names.iter()
                .filter(|name| !trimmed.is_empty() && name.starts_with(trimmed))
                .map(String::as_str)
                .collect();
            if matches.len() == 1 {
                app.input = format!("{} ", matches[0]);
                app.cursor_pos = app.input.len();
            } else if matches.len() > 1 {
                let common = longest_common_prefix(&matches);
                if common.len() > trimmed.len() {
                    app.input = common.clone();
                    app.cursor_pos = common.len();
                }
            }
        }
        _ => {}
    }
}

#[cfg(feature = "tui")]
fn longest_common_prefix(strs: &[&str]) -> String {
    if strs.is_empty() { return String::new(); }
    let mut prefix = strs[0].to_string();
    for s in &strs[1..] {
        while !s.starts_with(&prefix) {
            prefix.pop();
        }
    }
    prefix
}

#[cfg(feature = "tui")]
fn extract_tool_calls(messages: &[crate::provider::LlmMessage]) -> Option<Vec<serde_json::Value>> {
    let calls: Vec<serde_json::Value> = messages.iter().rev().filter_map(|m| match m {
        crate::provider::LlmMessage::AssistantWithReasoning { tool_calls, .. } if !tool_calls.is_empty() => {
            Some(tool_calls.iter().map(|tc| serde_json::json!({
                "id": tc.id, "name": tc.name, "args": tc.args
            })).collect::<Vec<_>>())
        }
        _ => None,
    }).next().unwrap_or_default();
    if calls.is_empty() { None } else { Some(calls) }
}

#[cfg(feature = "tui")]
async fn run_streaming_agent(
    config: &crate::config::Config,
    prompt: &str,
    event_tx: mpsc::Sender<AgentEvent>,
    history: Vec<crate::provider::LlmMessage>,
    cancel_rx: tokio::sync::oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let provider = crate::provider::create_provider(config)?;
    let tools = crate::tools::ToolRegistry::new(config)?;
    let mut agent = crate::agent::Agent::new(config.clone(), provider, tools, false);
    tokio::select! {
        result = agent.run_once_streaming(prompt, event_tx.clone(), history) => {
            result?;
            Ok(())
        }
        _ = cancel_rx => {
            event_tx.send(AgentEvent::Error("Cancelled by user".into())).await.ok();
            Ok(())
        }
    }
}

#[cfg(not(feature = "tui"))]
pub async fn run(_app: crate::app::App) -> anyhow::Result<()> {
    anyhow::bail!("TUI feature not enabled. Build with --features tui")
}

#[cfg(feature = "tui")]
fn expand_file_refs(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;
    while let Some(at_pos) = remaining.find('@') {
        result.push_str(&remaining[..at_pos]);
        let after = &remaining[at_pos + 1..];
        let end = after.find(|c: char| c.is_whitespace() || c == '\n').unwrap_or(after.len());
        let path = &after[..end];
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let preview: String = content.chars().take(3000).collect();
                result.push_str(&format!("\n[File: {}]\n```\n{}\n```\n", path, preview));
            }
            Err(_) => {
                result.push('@');
                result.push_str(path);
            }
        }
        remaining = &after[end..];
    }
    result.push_str(remaining);
    result
}
