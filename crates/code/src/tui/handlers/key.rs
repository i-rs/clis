use crate::app::{AgentMessage, App, AppMode};
use crate::agent::event::AgentEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

pub async fn handle_key(key: KeyEvent, app: &mut App, event_tx: &mpsc::Sender<AgentEvent>) {
    app.needs_redraw = true;
    if app.show_shortcuts {
        app.show_shortcuts = false;
        return;
    }

    if app.show_theme_picker {
        use crate::tui::colors::{set_active, THEMES};
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.show_theme_picker = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.theme_picker_selected > 0 {
                    app.theme_picker_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.theme_picker_selected + 1 < THEMES.len() {
                    app.theme_picker_selected += 1;
                }
            }
            KeyCode::Enter => {
                let selected = THEMES
                    .get(app.theme_picker_selected.min(THEMES.len() - 1))
                    .copied();
                if let Some(theme) = selected {
                    set_active(theme);
                    app.push_message(AgentMessage::system(format!(
                        "✓ Switched to theme: {} ({})",
                        theme.display_name, theme.id
                    )));
                }
                app.show_theme_picker = false;
            }
            _ => {}
        }
        app.needs_redraw = true;
        return;
    }

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
                        app.push_message(AgentMessage::Assistant {
                            content: format!("{}\n\n[Cancelled]", content),
                            reasoning,
                            tool_calls: None,
                            reasoning_expanded: false,
                            duration_ms: 0,
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

    if app.show_slash_picker
        && (key.code == KeyCode::Enter
            || key.code == KeyCode::Char('\r')
            || key.code == KeyCode::Char('\n'))
        && matches!(app.mode, AppMode::Idle)
    {
        let commands = super::super::ui::filtered_slash_commands(app);
        if let Some(selected) = commands.get(app.slash_selected) {
            let cmd_str = format!("/{}", selected.name);
            app.show_slash_picker = false;
            app.input.clear();
            app.input.push_history(&cmd_str);
            match crate::tui::slash_command::parse(&cmd_str) {
                Ok(cmd) => {
                    let msgs = crate::tui::slash_command::execute(cmd, app).await;
                    app.extend_messages(msgs);
                    app.auto_scroll = true;
                }
                Err(e) => {
                    app.push_message(AgentMessage::system(e));
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
        match crate::tui::slash_command::parse(&prompt) {
            Ok(cmd) => {
                let msgs = crate::tui::slash_command::execute(cmd, app).await;
                app.extend_messages(msgs);
                app.auto_scroll = true;
            }
            Err(e) => {
                app.push_message(AgentMessage::system(e));
                app.auto_scroll = true;
            }
        }
        app.needs_redraw = true;
        return;
    }

    match key.code {
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
                let result = if content.is_empty() {
                    // File didn't exist before write — delete it
                    tokio::fs::remove_file(&path).await
                } else {
                    tokio::fs::write(&path, &content).await
                };
                match result {
                    Ok(_) => {
                        app.push_message(AgentMessage::system(format!("Reverted {}", path)));
                    }
                    Err(e) => {
                        app.push_message(AgentMessage::system(format!(
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
                            app.push_message(AgentMessage::system(format!(
                                "Reverted {} files to session start (git stash)",
                                files
                            )));
                        } else {
                            app.push_message(AgentMessage::system(format!(
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
                        app.push_message(AgentMessage::system(
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
            let count = super::super::ui::filtered_slash_commands(app).len();
            if count > 0 {
                app.slash_selected = (app.slash_selected + 1).min(count.saturating_sub(1));
                app.needs_redraw = true;
            }
        }
        KeyCode::Up if matches!(app.mode, AppMode::Idle) && !app.input.history.is_empty() && !app.show_slash_picker => {
            app.input.history_up();
        }
        KeyCode::Down if matches!(app.mode, AppMode::Idle) && app.input.history_index.is_some() && !app.show_slash_picker => {
            app.input.history_down();
        }
        KeyCode::Up => app.scroll_up(),
        KeyCode::Down => app.scroll_down(),
        KeyCode::PageUp => app.scroll_offset = app.scroll_offset.saturating_sub(10),
        KeyCode::PageDown => app.scroll_offset = app.scroll_offset.saturating_add(10),
        KeyCode::Enter if matches!(app.mode, AppMode::Idle)
            && app.input.content.is_empty()
            && app.selected_message.is_some() =>
        {
            let idx = app.selected_message.unwrap_or(app.messages.len().saturating_sub(1));
            if idx < app.components.len() {
                use crate::tui::ui::components::ComponentOp;
                app.components[idx].borrow_mut().apply(ComponentOp::Toggle);
                app.layout_gen += 1;
                app.needs_redraw = true;
            }
        }
        KeyCode::Enter if key.modifiers == KeyModifiers::ALT => {
            app.input.insert_char('\n');
            app.needs_redraw = true;
        }
        KeyCode::Enter if matches!(app.mode, AppMode::Idle) && !app.input.content.is_empty() => {
            let prompt = std::mem::take(&mut app.input.content);
            app.input.push_history(&prompt);
            let expanded = super::super::expand_file_refs(&prompt).await;
            app.input.cursor_pos = 0;
            app.push_message(AgentMessage::user(&prompt));
            app.start_streaming();
            app.mode = AppMode::Waiting;

            let config = app.config.clone();
            let tx = event_tx.clone();
            let history = std::mem::take(&mut app.agent_messages);
            let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
            app.cancel_tx = Some(cancel_tx);
            app.task_handle = Some(tokio::spawn(async move {
                if let Err(e) =
                    super::super::run_streaming_agent(&config, &expanded, tx.clone(), history, cancel_rx).await
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
