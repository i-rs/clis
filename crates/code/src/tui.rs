#[cfg(feature = "tui")]
pub mod colors;
#[cfg(feature = "tui")]
pub mod highlight;
#[cfg(feature = "tui")]
pub mod input;
#[cfg(feature = "tui")]
pub mod overlays;
#[cfg(feature = "tui")]
pub mod sidebar;
#[cfg(feature = "tui")]
pub mod slash_command;
#[cfg(feature = "tui")]
pub mod strings;
#[cfg(feature = "tui")]
pub mod transcript;
#[cfg(feature = "tui")]
pub mod ui;
#[cfg(feature = "tui")]
pub mod utils;

#[cfg(feature = "tui")]
use crate::agent::event::AgentEvent;
#[cfg(feature = "tui")]
use crate::app::{AgentMessage, App, AppMode, ToolCallInfo};
#[cfg(feature = "tui")]
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
#[cfg(feature = "tui")]
use ratatui::{Terminal, backend::CrosstermBackend};
#[cfg(feature = "tui")]
use std::io;
#[cfg(feature = "tui")]
use std::time::Duration;
#[cfg(feature = "tui")]
use tokio::sync::mpsc;

#[cfg(feature = "tui")]
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
        app.messages.push(AgentMessage::Assistant {
            content: format!(
                "Welcome to i-rs-code v{version}\n\n\
                 Type a message to start coding...\n\n\
                 Slash commands:\n  \
                 /help  Show all available commands\n\n\
                 Other commands:\n  \
                 i-rs-code chat <prompt>  One-shot conversation\n  \
                 i-rs-code config init    Interactive setup\n  \
                 i-rs-code config show    View configuration"
            ),
            reasoning: String::new(),
            tool_calls: None,
            reasoning_expanded: false,
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
        terminal.draw(|f| {
            if app.show_transcript {
                transcript::render_transcript(f, &app);
            } else {
                ui::render(f, &app);
            }
        })?;

        // Faster poll (15ms) when streaming so tokens render sooner
        let poll_ms = if app.streaming.is_some() { 15 } else { 50 };
        if event::poll(Duration::from_millis(poll_ms))? {
            match event::read()? {
                Event::Key(key) => {
                    handle_key(key, &mut app, &event_tx).await;
                }
                Event::Paste(data) => {
                    for c in data.chars() {
                        app.input.insert_char(c);
                    }
                }
                Event::Mouse(mouse) => {
                    let sidebar_width = 38u16;
                    let is_sidebar = terminal
                        .size()
                        .map(|s| mouse.column > s.width.saturating_sub(sidebar_width))
                        .unwrap_or(false);
                    match mouse.kind {
                        MouseEventKind::ScrollUp => {
                            if is_sidebar {
                                app.sidebar_scroll = app.sidebar_scroll.saturating_sub(3);
                            } else {
                                app.scroll_offset = app.scroll_offset.saturating_sub(3);
                                app.auto_scroll = false;
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if is_sidebar {
                                app.sidebar_scroll = app.sidebar_scroll.saturating_add(3);
                            } else {
                                app.scroll_offset = app.scroll_offset.saturating_add(3);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        while let Ok(event) = event_rx.try_recv() {
            handle_event(event, &mut app).await;
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

#[cfg(feature = "tui")]
async fn handle_event(event: AgentEvent, app: &mut App) {
    app.needs_redraw = true;
    match event {
        AgentEvent::Token(t) => {
            if let Some(ref mut s) = app.streaming {
                s.content.push_str(&t);
            }
        }
        AgentEvent::Reasoning(r) => {
            if let Some(ref mut s) = app.streaming {
                s.reasoning.push_str(&r);
            }
        }
        AgentEvent::ToolCallStart {
            id: _id,
            name,
            args,
        } => {
            let path_key = args
                .get("file_path")
                .or_else(|| args.get("path"))
                .and_then(|v| v.as_str());
            if matches!(name.as_str(), "write" | "edit" | "delete")
                && let Some(path) = path_key
            {
                let rel = make_relative(&app.current_dir, path);
                app.file_changes.insert(rel.clone());
                if !matches!(name.as_str(), "delete")
                    && let Ok(content) = tokio::fs::read_to_string(path).await
                {
                    app.last_file_states.push((rel, content));
                }
            }
            if name.as_str() == "rename"
                && let Some(from) = args.get("from").and_then(|v| v.as_str())
            {
                app.file_changes
                    .insert(make_relative(&app.current_dir, from));
                if let Some(to) = args.get("to").and_then(|v| v.as_str()) {
                    app.file_changes.insert(make_relative(&app.current_dir, to));
                }
            }
            let diff = if name.as_str() == "edit" {
                compute_diff_from_args(&args)
            } else {
                None
            };
            let info = ToolCallInfo {
                name,
                args: serde_json::to_string_pretty(&args).unwrap_or_default(),
                result: None,
                diff,
            };
            if let Some(ref mut s) = app.streaming {
                s.current_tool = Some(info);
            }
        }
        AgentEvent::ToolCallEnd {
            id: _id,
            name: _name,
            result,
        } => {
            if let Some(ref mut s) = app.streaming
                && let Some(mut tool) = s.current_tool.take()
            {
                tool.result = Some(result);
                s.tool_calls.push(tool);
            }
        }
        AgentEvent::Status(msg) => {
            app.status_message = Some(msg);
        }
        AgentEvent::Plan { steps } => {
            app.plan = steps;
        }
        AgentEvent::Done {
            usage,
            messages,
            context_pct,
        } => {
            app.status_message = None;
            let streamed_tc = app
                .streaming
                .as_ref()
                .map(|s| s.tool_calls.clone())
                .unwrap_or_default();
            let (content, reasoning) = app.finish_streaming();

            for tc in &streamed_tc {
                let display = match &tc.result {
                    Some(r) => {
                        let preview: String = r.chars().take(2000).collect();
                        if preview.len() < r.len() {
                            format!("{}\n{}...", tc.name, preview)
                        } else {
                            format!("{}\n{}", tc.name, preview)
                        }
                    }
                    None => format!("{}\n(no result)", tc.name),
                };
                app.messages.push(AgentMessage::ToolResult {
                    content: format!("\n{}", display),
                    diff: tc.diff.clone(),
                });
            }

            if !messages.is_empty() {
                app.agent_messages = messages.clone();
            }
            if let Some(u) = usage {
                app.add_token_usage(u.input_tokens, u.output_tokens);
            }
            app.context_usage = Some(context_pct);
            let tool_calls = extract_tool_calls(&messages);
            if !content.is_empty() || !reasoning.is_empty() || tool_calls.is_some() {
                app.messages.push(AgentMessage::Assistant {
                    content,
                    reasoning,
                    tool_calls,
                    reasoning_expanded: false,
                });
            }

            // Completion summary
            if !app.file_changes.is_empty() {
                let mut files: Vec<&String> = app.file_changes.iter().collect();
                files.sort();
                let mut summary = format!("── 完成 ──\n📄 {} 个文件:", files.len());
                for f in &files {
                    summary.push_str(&format!("\n  {}", f));
                }
                if !streamed_tc.is_empty() {
                    let mut tool_counts: std::collections::BTreeMap<&str, usize> =
                        std::collections::BTreeMap::new();
                    for tc in &streamed_tc {
                        *tool_counts.entry(tc.name.as_str()).or_insert(0) += 1;
                    }
                    let tool_str: Vec<String> = tool_counts
                        .iter()
                        .map(|(n, c)| format!("{} ×{}", n, c))
                        .collect();
                    summary.push_str(&format!("\n🔧 {}", tool_str.join("  ")));
                }
                app.messages.push(AgentMessage::system(summary));
            }

            // Separator
            app.messages.push(AgentMessage::Separator {
                label: String::new(),
            });

            // scroll_offset preserved when auto_scroll=false (absolute line semantics)
            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
        AgentEvent::Error(e) => {
            app.status_message = None;
            let (content, reasoning) = app.finish_streaming();
            let msg = if !content.is_empty() {
                format!("{}\n\nError: {}", content, e)
            } else {
                format!("Error: {}", e)
            };
            app.messages.push(AgentMessage::Assistant {
                content: msg,
                reasoning,
                tool_calls: None,
                reasoning_expanded: false,
            });
            if matches!(app.mode, AppMode::Waiting) {
                app.mode = AppMode::Idle;
            }
        }
    }
}

#[cfg(feature = "tui")]
async fn handle_key(key: KeyEvent, app: &mut App, event_tx: &mpsc::Sender<AgentEvent>) {
    app.needs_redraw = true;
    if app.show_shortcuts {
        app.show_shortcuts = false;
        return;
    }

    // Transcript mode key handling
    if app.show_transcript {
        match key.code {
            KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                app.show_transcript = false;
                app.transcript_scroll = 0;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.show_transcript = false;
                app.transcript_scroll = 0;
            }
            KeyCode::Up => app.transcript_scroll = app.transcript_scroll.saturating_sub(1),
            KeyCode::Down => app.transcript_scroll = app.transcript_scroll.saturating_add(1),
            KeyCode::PageUp => app.transcript_scroll = app.transcript_scroll.saturating_sub(10),
            KeyCode::PageDown => app.transcript_scroll = app.transcript_scroll.saturating_add(10),
            KeyCode::Home => app.transcript_scroll = 0,
            KeyCode::End => app.transcript_scroll = usize::MAX,
            _ => {}
        }
        return;
    }

    match app.mode {
        AppMode::Waiting => {
            match key.code {
                KeyCode::Up if key.modifiers == KeyModifiers::CONTROL => {
                    app.sidebar_scroll = app.sidebar_scroll.saturating_sub(1);
                    return;
                }
                KeyCode::Down if key.modifiers == KeyModifiers::CONTROL => {
                    app.sidebar_scroll = app.sidebar_scroll.saturating_add(1);
                    return;
                }
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
                        app.messages.push(AgentMessage::Assistant {
                            content: format!("{}\n\n[Cancelled]", content),
                            reasoning,
                            tool_calls: None,
                            reasoning_expanded: false,
                        });
                    }
                    return;
                }
                KeyCode::Up => app.scroll_up(),
                KeyCode::Down => app.scroll_down(),
                KeyCode::PageUp => app.scroll_offset = app.scroll_offset.saturating_sub(10),
                KeyCode::PageDown => app.scroll_offset = app.scroll_offset.saturating_add(10),
                _ => {}
            }
            return;
        }
        AppMode::Idle => {
            if key.code == KeyCode::Up && key.modifiers == KeyModifiers::CONTROL {
                app.sidebar_scroll = app.sidebar_scroll.saturating_sub(1);
                return;
            }
            if key.code == KeyCode::Down && key.modifiers == KeyModifiers::CONTROL {
                app.sidebar_scroll = app.sidebar_scroll.saturating_add(1);
                return;
            }
        }
    }

    // Slash commands: intercept BEFORE the main key match to guarantee local execution.
    // These are local commands (/clear, /help, /model, etc.) and MUST NOT be sent to the LLM.
    // Check both KeyCode::Enter and KeyCode::Char('\r')/('\n') for terminal compatibility.
    if app.show_slash_picker
        && (key.code == KeyCode::Enter
            || key.code == KeyCode::Char('\r')
            || key.code == KeyCode::Char('\n'))
        && matches!(app.mode, AppMode::Idle)
    {
        let commands = ui::filtered_slash_commands(app);
        if let Some(selected) = commands.get(app.slash_selected) {
            let cmd_str = format!("/{}", selected.name);
            app.show_slash_picker = false;
            app.input.clear();
            app.input.push_history(&cmd_str);
            match slash_command::parse(&cmd_str) {
                Ok(cmd) => {
                    let msgs = slash_command::execute(cmd, app).await;
                    app.messages.extend(msgs);
                    app.auto_scroll = true;
                }
                Err(e) => {
                    app.messages.push(AgentMessage::system(e));
                    app.auto_scroll = true;
                }
            }
        } else {
            app.show_slash_picker = false;
            app.input.clear();
        }
        app.needs_redraw = true;
        return;
    }

    if (key.code == KeyCode::Enter
        || key.code == KeyCode::Char('\r')
        || key.code == KeyCode::Char('\n'))
        && matches!(app.mode, AppMode::Idle)
        && !app.show_slash_picker
        && !app.input.content.is_empty()
        && app.input.content.trim().starts_with('/')
    {
        app.show_slash_picker = false;
        let prompt = std::mem::take(&mut app.input.content);
        app.input.push_history(&prompt);
        app.input.cursor_pos = 0;
        match slash_command::parse(&prompt) {
            Ok(cmd) => {
                let msgs = slash_command::execute(cmd, app).await;
                app.messages.extend(msgs);
                app.auto_scroll = true;
            }
            Err(e) => {
                app.messages.push(AgentMessage::system(e));
                app.auto_scroll = true;
            }
        }
        app.needs_redraw = true;
        return;
    }

    match key.code {
        KeyCode::Char('r') if matches!(app.mode, AppMode::Idle) && app.input.content.is_empty() => {
            // If a message is selected, toggle that one; otherwise toggle the last assistant message
            if let Some(idx) = app.selected_message
                && let Some(AgentMessage::Assistant {
                    reasoning_expanded, ..
                }) = app.messages.get_mut(idx)
            {
                *reasoning_expanded = !*reasoning_expanded;
            } else {
                for msg in app.messages.iter_mut().rev() {
                    if let AgentMessage::Assistant {
                        reasoning_expanded, ..
                    } = msg
                    {
                        *reasoning_expanded = !*reasoning_expanded;
                        break;
                    }
                }
            }
        }
        KeyCode::Char('[') if matches!(app.mode, AppMode::Idle) && app.input.content.is_empty() => {
            let idx = app.selected_message.unwrap_or(app.messages.len());
            app.selected_message = Some(idx.saturating_sub(1));
        }
        KeyCode::Char(']') if matches!(app.mode, AppMode::Idle) && app.input.content.is_empty() => {
            let idx = app.selected_message.unwrap_or(usize::MAX);
            let next = idx.saturating_add(1);
            app.selected_message = if next < app.messages.len() {
                Some(next)
            } else {
                Some(app.messages.len().saturating_sub(1))
            };
        }
        KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
            app.show_transcript = !app.show_transcript;
            app.transcript_scroll = 0;
        }
        KeyCode::Char('z') if key.modifiers == KeyModifiers::CONTROL => {
            if let Some((path, content)) = app.last_file_states.pop() {
                match tokio::fs::write(&path, &content).await {
                    Ok(_) => {
                        app.messages
                            .push(AgentMessage::system(format!("Reverted {}", path)));
                    }
                    Err(e) => {
                        app.messages.push(AgentMessage::system(format!(
                            "Failed to revert {}: {}",
                            path, e
                        )));
                    }
                }
            } else if let Some((commit_msg, files)) = app.git_baseline.take() {
                let result = tokio::process::Command::new("git")
                    .args(["stash", "push", "-m", &commit_msg])
                    .output()
                    .await;
                match result {
                    Ok(output) => {
                        if output.status.success() {
                            app.messages.push(AgentMessage::system(format!(
                                "Reverted {} files to session start (git stash)",
                                files
                            )));
                        } else {
                            app.messages.push(AgentMessage::system(format!(
                                "Git stash failed: {}",
                                String::from_utf8_lossy(&output.stderr)
                            )));
                        }
                    }
                    Err(e) => {
                        app.messages
                            .push(AgentMessage::system(format!("Git stash failed: {}", e)));
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
            } else if app.show_slash_picker {
                app.show_slash_picker = false;
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('?') if !app.show_debug => {
            app.show_shortcuts = !app.show_shortcuts;
        }
        KeyCode::Char('b') if key.modifiers == KeyModifiers::CONTROL => {
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
            if key.modifiers == KeyModifiers::CONTROL {
                match c {
                    'c' if matches!(app.mode, AppMode::Idle) => {
                        app.messages.push(AgentMessage::system(
                            "按 Esc 或 q 退出。Ctrl+C 不能退出，Ctrl+B 打开调试面板。",
                        ));
                        return;
                    }
                    'a' | 'A' => {
                        app.input.cursor_pos = 0;
                        return;
                    }
                    'e' | 'E' => {
                        app.input.cursor_pos = app.input.content.len();
                        return;
                    }
                    'u' | 'U' => {
                        app.input.clear();
                        return;
                    }
                    _ => {}
                }
            }
            app.input.insert_char(c);
            app.needs_redraw = true;
        }
        KeyCode::Backspace => {
            app.input.delete_char();
            app.needs_redraw = true;
        }
        KeyCode::Delete if app.input.cursor_pos < app.input.content.len() => {
            app.input.delete_forward();
            app.needs_redraw = true;
        }
        KeyCode::Left => {
            app.input.move_left();
            app.needs_redraw = true;
        }
        KeyCode::Right => {
            app.input.move_right();
            app.needs_redraw = true;
        }
        KeyCode::Home => {
            app.input.cursor_pos = 0;
            app.needs_redraw = true;
        }
        KeyCode::End => {
            app.input.cursor_pos = app.input.content.len();
            app.needs_redraw = true;
        }
        KeyCode::Up if app.show_slash_picker => {
            app.slash_selected = app.slash_selected.saturating_sub(1);
            app.needs_redraw = true;
        }
        KeyCode::Down if app.show_slash_picker => {
            let count = ui::filtered_slash_commands(app).len();
            if count > 0 {
                app.slash_selected = (app.slash_selected + 1).min(count.saturating_sub(1));
                app.needs_redraw = true;
            }
        }
        KeyCode::Up if matches!(app.mode, AppMode::Idle) && app.input.content.is_empty() => {
            app.input.history_up();
        }
        KeyCode::Down if matches!(app.mode, AppMode::Idle) && app.input.content.is_empty() => {
            app.input.history_down();
        }
        KeyCode::Up => app.scroll_up(),
        KeyCode::Down => app.scroll_down(),
        KeyCode::PageUp => app.scroll_offset = app.scroll_offset.saturating_sub(10),
        KeyCode::PageDown => app.scroll_offset = app.scroll_offset.saturating_add(10),
        KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
            app.input.insert_char('\n');
            app.needs_redraw = true;
        }
        KeyCode::Enter if matches!(app.mode, AppMode::Idle) && !app.input.content.is_empty() => {
            let prompt = std::mem::take(&mut app.input.content);
            app.input.push_history(&prompt);
            let expanded = expand_file_refs(&prompt).await;
            app.input.cursor_pos = 0;
            app.messages.push(AgentMessage::user(&prompt));
            app.start_streaming();
            app.mode = AppMode::Waiting;

            let config = app.config.clone();
            let tx = event_tx.clone();
            let history = std::mem::take(&mut app.agent_messages);
            let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
            app.cancel_tx = Some(cancel_tx);
            app.task_handle = Some(tokio::spawn(async move {
                if let Err(e) =
                    run_streaming_agent(&config, &expanded, tx.clone(), history, cancel_rx).await
                {
                    tx.send(AgentEvent::Error(e.to_string())).await.ok();
                }
            }));
        }
        KeyCode::Tab => {
            let input = &app.input.content;
            let trimmed = input.trim();
            if let Some(stripped) = trimmed.strip_prefix('/') {
                let partial = stripped.to_lowercase();
                let cmd_names: Vec<&str> = crate::tui::slash_command::COMMANDS
                    .iter()
                    .map(|c| c.name)
                    .filter(|name| name.starts_with(&partial))
                    .collect();
                if cmd_names.len() == 1 {
                    app.input.content = format!("/{} ", cmd_names[0]);
                    app.input.cursor_pos = app.input.content.len();
                } else if cmd_names.len() > 1 {
                    let common = longest_common_prefix(&cmd_names);
                    if common.len() > partial.len() {
                        app.input.content = format!("/{}", common);
                        app.input.cursor_pos = app.input.content.len();
                    }
                }
            } else {
                let matches: Vec<&str> = app
                    .tool_names
                    .iter()
                    .filter(|name| !trimmed.is_empty() && name.starts_with(trimmed))
                    .map(String::as_str)
                    .collect();
                if matches.len() == 1 {
                    app.input.content = format!("{} ", matches[0]);
                    app.input.cursor_pos = app.input.content.len();
                } else if matches.len() > 1 {
                    let common = longest_common_prefix(&matches);
                    if common.len() > trimmed.len() {
                        app.input.content = common.clone();
                        app.input.cursor_pos = common.len();
                    }
                }
            }
        }
        _ => {}
    }

    // Update slash picker visibility based on input content
    if matches!(app.mode, AppMode::Idle) {
        let starts_with_slash = !app.input.content.is_empty() && app.input.content.starts_with('/');
        if starts_with_slash && !app.show_slash_picker {
            app.show_slash_picker = true;
            app.slash_selected = 0;
        } else if !starts_with_slash && app.show_slash_picker {
            app.show_slash_picker = false;
        }
    }
}

#[cfg(feature = "tui")]
fn longest_common_prefix(strs: &[&str]) -> String {
    if strs.is_empty() {
        return String::new();
    }
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
fn make_relative(base: &str, path: &str) -> String {
    use std::path::Path;
    Path::new(path)
        .strip_prefix(base)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.to_string())
}

#[cfg(feature = "tui")]
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
