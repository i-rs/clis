pub mod colors;
pub mod handlers;
pub mod highlight;
pub mod input;
pub mod overlays;
pub mod sidebar;
pub mod slash_command;
pub mod strings;
pub mod transcript;
pub mod ui;
pub mod utils;

use crate::agent::event::AgentEvent;
use crate::app::{AgentMessage, App};
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

pub async fn run(mut app: App) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(1024);

    if app.messages.is_empty() {
        let version = app.version.clone();
        app.push_message(AgentMessage::Assistant {
            content: format!(
                "# Welcome to i-rs-code v{version}\n\n\
                 Type a message to start coding.\n\n\
                 ## Slash commands\n\
                 - /help  Show all available commands\n\n\
                 ## Other commands\n\
                 - `i-rs-code chat <prompt>`  One-shot conversation\n\
                 - `i-rs-code config init`    Interactive setup\n\
                 - `i-rs-code config show`    View configuration"
            ),
            reasoning: String::new(),
            tool_calls: None,
            reasoning_expanded: false,
            duration_ms: 0,
        });
    }

    if app.git_baseline.is_none() {
        let git_output = tokio::process::Command::new("git")
            .args(["diff", "--stat"])
            .current_dir(&app.current_dir)
            .output()
            .await;
        if let Ok(out) = git_output
            && out.status.success()
        {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let file_count = stderr
                .lines()
                .find(|l| l.contains(" file"))
                .and_then(|l| l.split_whitespace().next())
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(0);
            if file_count > 0 {
                app.git_baseline = Some((
                    format!(
                        "i-rs-code-undo-{}",
                        chrono::Utc::now().format("%Y%m%d%H%M%S")
                    ),
                    file_count,
                ));
            }
        }
    }

    while !app.should_quit {
        let is_streaming = app.streaming.is_some();
        if app.needs_redraw || is_streaming {
            terminal.draw(|f| {
            if app.show_transcript {
                transcript::render_transcript(f, &app);
            } else if app.show_theme_picker {
                use crate::tui::colors::THEMES;
                let total = THEMES.len();
                let selected = app.theme_picker_selected.min(total.saturating_sub(1));
                if let Some(&preview) = THEMES.get(selected) {
                    crate::tui::colors::with_preview(preview, || {
                        ui::render(f, &app);
                    });
                } else {
                    ui::render(f, &app);
                }
            } else {
                ui::render(f, &app);
            }
            })?;
            if !app.auto_scroll {
                let max_scroll = ui::get_max_scroll();
                if app.scroll_offset >= max_scroll {
                    app.auto_scroll = true;
                }
                app.scroll_offset = app.scroll_offset.min(max_scroll);
            }
            app.needs_redraw = false;
        }

        let poll_ms = if is_streaming { 16 } else { 50 };
        if event::poll(Duration::from_millis(poll_ms))? {
            match event::read()? {
                Event::Key(key) => {
                    handlers::handle_key(key, &mut app, &event_tx).await;
                }
                Event::Paste(data) => {
                    for c in data.chars() {
                        app.input.insert_char(c);
                    }
                }
                Event::Mouse(mouse) => {
                    let terminal_size = terminal.size().unwrap_or_default();
                    let sidebar_width = 40u16;
                    let is_sidebar =
                        mouse.column > terminal_size.width.saturating_sub(sidebar_width);
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            if is_sidebar {
                                app.sidebar_scroll = app.sidebar_scroll.saturating_sub(1);
                            } else {
                                app.scroll_offset = app.scroll_offset.saturating_sub(1);
                                app.auto_scroll = false;
                                app.needs_redraw = true;
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if is_sidebar {
                                app.sidebar_scroll = app.sidebar_scroll.saturating_add(1);
                            } else {
                                app.scroll_offset = app.scroll_offset.saturating_add(1);
                                app.needs_redraw = true;
                            }
                        }
                        MouseEventKind::Down(_)
                            if !is_sidebar && mouse.row > 0 =>
                        {
                            let hint_shown = !app.auto_scroll && app.messages.len() > 1;
                            let chat_area_y = 1u16; // title bar height
                            if ui::streaming_click_target(mouse.row, app.scroll_offset, hint_shown, chat_area_y) {
                                if let Some(ref mut s) = app.streaming {
                                    s.reasoning_collapsed = !s.reasoning_collapsed;
                                    app.needs_redraw = true;
                                }
                            } else if let Some(op) = ui::click_op_at_screen(
                                mouse.row, app.scroll_offset, hint_shown, chat_area_y, &app.components
                            ) {
                                // Forward to component via click op (checks extra_click_targets)
                                let idx = ui::find_message_idx_from_screen(
                                    mouse.row, app.scroll_offset, hint_shown, chat_area_y
                                );
                                if let Some(idx) = idx {
                                    app.components[idx].borrow_mut().apply(op);
                                    app.layout_gen += 1;
                                    app.needs_redraw = true;
                                    app.selected_message = Some(idx);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        while let Ok(event) = event_rx.try_recv() {
            handlers::handle_event(event, &mut app).await;
        }
    }

    let stats = if app
        .messages
        .iter()
        .any(|m| matches!(m, AgentMessage::User { .. }))
    {
        let tool_count: usize = app
            .messages
            .iter()
            .filter(|m| matches!(m, AgentMessage::ToolResult { .. }))
            .count();
        let session_id = app
            .session_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let sessions_dir = crate::config::i_rs_code_dir().join("sessions");
        let session = crate::session::Session::from_agent_messages(
            Some(session_id.clone()),
            &app.messages,
            std::mem::take(&mut app.agent_messages),
        );
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
        DisableBracketedPaste,
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

#[cfg(not(feature = "tui"))]
pub async fn run(_app: crate::app::App) -> anyhow::Result<()> {
    anyhow::bail!("TUI feature not enabled. Build with --features tui")
}

// ═══════════════════════════════════════════════════════════════════════════
//  Helper functions used by handlers (via super::super::*)
// ═══════════════════════════════════════════════════════════════════════════

fn make_relative(base: &str, path: &str) -> String {
    use std::path::Path;
    Path::new(path)
        .strip_prefix(base)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.to_string())
}

async fn expand_file_refs(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;
    while let Some(at_pos) = remaining.find('@') {
        result.push_str(&remaining[..at_pos]);
        let after = &remaining[at_pos + 1..];
        let end = after
            .find(|c: char| c.is_whitespace() || c == '\n')
            .unwrap_or(after.len());
        let path = &after[..end];
        match tokio::fs::read_to_string(path).await {
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

fn extract_tool_calls(messages: &[crate::provider::LlmMessage]) -> Option<Vec<serde_json::Value>> {
    let calls: Vec<serde_json::Value> = messages
        .iter()
        .rev()
        .filter_map(|m| match m {
            crate::provider::LlmMessage::AssistantWithReasoning { tool_calls, .. }
                if !tool_calls.is_empty() =>
            {
                Some(
                    tool_calls
                        .iter()
                        .map(|tc| {
                            serde_json::json!({
                                "id": tc.id, "name": tc.name, "args": tc.args
                            })
                        })
                        .collect::<Vec<_>>(),
                )
            }
            _ => None,
        })
        .next()
        .unwrap_or_default();
    if calls.is_empty() { None } else { Some(calls) }
}

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

fn compute_diff_from_args(args: &serde_json::Value) -> Option<String> {
    let old = args.get("old_string").and_then(|v| v.as_str())?;
    let new = args.get("new_string").and_then(|v| v.as_str())?;
    if old == new {
        return None;
    }
    let diff = crate::diff::diff_text(old, new);
    let line_count = diff.patch.lines().count();
    if line_count <= 20 {
        return Some(diff.patch);
    }
    let mut result = String::with_capacity(diff.patch.len().min(400) + 24);
    for line in diff.patch.lines().take(20) {
        result.push_str(line);
        result.push('\n');
    }
    result.push_str("... (diff truncated)");
    Some(result)
}
