use crate::app::{App, Overlay};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json::Value;

use super::{Action, AppMessage, KeyEventHandler};

pub fn handle_active_overlay_keys(
    handler: &mut KeyEventHandler,
    key: KeyEvent,
    overlay: Overlay,
) -> Action {
    match overlay {
        Overlay::SessionList => handle_session_list_keys(handler, key),
        Overlay::Sidebar => handle_sidebar_keys(handler, key),
        Overlay::AgentPicker => handle_agent_picker_keys(handler, key),
        Overlay::ThemePicker => handle_theme_picker_keys(handler, key),
        _ => Action::Continue,
    }
}

fn handle_session_list_keys(handler: &mut KeyEventHandler, key: KeyEvent) -> Action {
    let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let filtered = handler.app.overlay.filtered_sessions_cached();
    let filtered_len = filtered.len();
    let filtered_max = filtered_len.saturating_sub(1);
    match key.code {
        KeyCode::Char('d') if has_ctrl => {
            handler.app.overlay.session_confirm_delete = true;
        }
        KeyCode::Char('r') if has_ctrl => {
            if let Some(meta) = filtered.get(handler.app.overlay.session_list_index) {
                handler.app.overlay.session_rename_buf = meta.title.clone();
            }
        }
        KeyCode::Char('y') if handler.app.overlay.session_confirm_delete => {
            if let Some(meta) = filtered.get(handler.app.overlay.session_list_index) {
                let id = meta.id.clone();
                let is_current = handler
                    .app_core
                    .session_mgr
                    .current_id()
                    .map(|cid| cid == id)
                    .unwrap_or(false);
                handler.app_core.session_mgr.delete_session(&id);
                handler.app.overlay.session_confirm_delete = false;
                handler.app.overlay.session_list = handler.app_core.session_mgr.sessions().to_vec();
                handler.app.overlay.invalidate_session_cache();
                if is_current {
                    handler.app.reset_for_new_session();
                }
            } else {
                handler.app.overlay.session_confirm_delete = false;
            }
        }
        KeyCode::Char('n') if handler.app.overlay.session_confirm_delete => {
            handler.app.overlay.session_confirm_delete = false;
        }
        KeyCode::Esc => {
            if handler.app.overlay.session_search_mode {
                handler.app.overlay.session_search_mode = false;
                handler.app.overlay.session_search.clear();
            } else if handler.app.overlay.session_confirm_delete {
                handler.app.overlay.session_confirm_delete = false;
            } else if !handler.app.overlay.session_rename_buf.is_empty() {
                handler.app.overlay.session_rename_buf.clear();
            } else {
                handler.app.overlay.close();
            }
        }
        KeyCode::Up => {
            handler.app.overlay.session_list_index =
                handler.app.overlay.session_list_index.saturating_sub(1);
        }
        KeyCode::Down => {
            if handler.app.overlay.session_list_index < filtered_max {
                handler.app.overlay.session_list_index += 1;
            }
        }
        KeyCode::PageUp => {
            let step = 10.min(handler.app.overlay.session_list_index);
            handler.app.overlay.session_list_index -= step;
        }
        KeyCode::PageDown => {
            handler.app.overlay.session_list_index =
                (handler.app.overlay.session_list_index + 10).min(filtered_max);
        }
        KeyCode::Home => {
            handler.app.overlay.session_list_index = 0;
        }
        KeyCode::End => {
            handler.app.overlay.session_list_index = filtered_max;
        }
        KeyCode::Char('/') if !handler.app.overlay.session_search_mode => {
            handler.app.overlay.session_search_mode = true;
            handler.app.overlay.session_search.clear();
        }
        KeyCode::Char(c) if !handler.app.overlay.session_rename_buf.is_empty() => {
            handler.app.overlay.session_rename_buf.push(c);
        }
        KeyCode::Char(c) if handler.app.overlay.session_search_mode => {
            handler.app.overlay.session_search.push(c);
            handler.app.overlay.session_list_index = 0;
        }
        KeyCode::Backspace
            if !handler.app.overlay.session_rename_buf.is_empty()
                && !handler.app.overlay.session_search_mode =>
        {
            handler.app.overlay.session_rename_buf.pop();
        }
        KeyCode::Backspace if handler.app.overlay.session_search_mode => {
            handler.app.overlay.session_search.pop();
            handler.app.overlay.session_list_index = 0;
        }
        KeyCode::Enter => {
            return handle_session_enter(handler, &filtered);
        }
        _ => {}
    }
    handler.app.mark_overlay_dirty();
    Action::Continue
}

fn handle_session_enter(
    handler: &mut KeyEventHandler,
    filtered: &[i_rs_claw_core::session::SessionMeta],
) -> Action {
    if !handler.app.overlay.session_rename_buf.is_empty() {
        if let Some(meta) = filtered.get(handler.app.overlay.session_list_index) {
            let title = std::mem::take(&mut handler.app.overlay.session_rename_buf);
            if !title.trim().is_empty() {
                handler
                    .app_core
                    .session_mgr
                    .rename_session(&meta.id, title.trim());
            }
            handler.app.overlay.session_list = handler.app_core.session_mgr.sessions().to_vec();
            handler.app.overlay.invalidate_session_cache();
            handler.app.overlay.session_rename_buf.clear();
        }
        handler.app.mark_overlay_dirty();
        return Action::Continue;
    }

    if handler.app.overlay.session_search_mode {
        handler.app.overlay.session_search_mode = false;
    }

    if let Some(meta) = filtered.get(handler.app.overlay.session_list_index) {
        let new_id = meta.id.clone();
        let is_current = handler
            .app_core
            .session_mgr
            .current_id()
            .map(|id| id == new_id)
            .unwrap_or(false);
        if !is_current {
            let old_id = handler
                .app_core
                .session_mgr
                .current_id()
                .unwrap_or_default()
                .to_string();
            crate::tui::clipboard::persist_session_messages(
                &mut handler.app_core.session_mgr,
                &old_id,
                &handler.app.chat.messages,
                handler.app.chat.api_messages.as_deref(),
            );

            handler.app_core.session_mgr.switch_to(&new_id);
            let loaded = handler.app_core.session_mgr.load_app_messages(&new_id, 200);
            handler.app.chat.messages = loaded;
            handler
                .app_core
                .session_mgr
                .reset_cursor(&new_id, handler.app.chat.messages.len());
            handler.app.sync_message_timestamps();
            handler.app.chat.api_messages = handler.app_core.session_mgr.load_api_messages(&new_id);
            handler.app.chat.tool_call_count = 0;
            handler.app.llm.status_text.clear();
            handler.app.llm.token_usage = None;
            handler.app.chat.plan_steps = handler.app_core.session_mgr.load_plan_steps(&new_id);
            // Snap to the bottom of the freshly loaded session: render
            // reads `stick_to_bottom` and overrides `scroll_lines` with
            // the current `max_scroll`.
            handler.app.scroll.scroll_lines = 0;
            handler.app.scroll.max_scroll = 0;
            handler.app.scroll.stick_to_bottom = true;
            // Component state is rebuilt from the freshly loaded
            // messages a moment later, so we don't need to clear the
            // hash sets here any more.
            handler.app.mark_dirty();
        }
    }
    handler.app.overlay.close();
    handler.app.mark_overlay_dirty();
    Action::Continue
}

fn handle_sidebar_keys(handler: &mut KeyEventHandler, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc if handler.app.overlay.sidebar_body_idx.is_some() => {
            handler.app.overlay.sidebar_body_idx = None;
            handler.app.overlay.sidebar_formatted_json = None;
        }
        KeyCode::Esc => {
            handler.app.overlay.close();
        }
        KeyCode::Up if handler.app.overlay.sidebar_body_idx.is_none() => {
            handler.app.overlay.sidebar_selected =
                handler.app.overlay.sidebar_selected.saturating_sub(1);
        }
        KeyCode::Down if handler.app.overlay.sidebar_body_idx.is_none() => {
            let max = handler.app.http_logs.len().saturating_sub(1);
            if handler.app.overlay.sidebar_selected < max {
                handler.app.overlay.sidebar_selected += 1;
            }
        }
        KeyCode::Enter
            if handler.app.overlay.sidebar_body_idx.is_none()
                && !handler.app.http_logs.is_empty() =>
        {
            handler.app.overlay.sidebar_body_idx = Some(handler.app.overlay.sidebar_selected);
            handler.app.overlay.sidebar_formatted_json = None;
            handler.app.overlay.sidebar_body_scroll = 0;
        }
        KeyCode::Up if handler.app.overlay.sidebar_body_idx.is_some() => {
            handler.app.overlay.sidebar_body_scroll =
                handler.app.overlay.sidebar_body_scroll.saturating_sub(3);
        }
        KeyCode::Down if handler.app.overlay.sidebar_body_idx.is_some() => {
            if let Some(idx) = handler.app.overlay.sidebar_body_idx
                && let Some(log) = handler.app.http_logs.get(idx)
            {
                let content_lines = log.request_body.lines().count() * 3;
                let max = content_lines.saturating_sub(1);
                handler.app.overlay.sidebar_body_scroll =
                    (handler.app.overlay.sidebar_body_scroll + 3).min(max);
            }
        }
        _ => {}
    }
    handler.app.mark_overlay_dirty();
    Action::Continue
}

fn handle_agent_picker_keys(handler: &mut KeyEventHandler, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => {
            handler.app.overlay.agent_picker_index =
                handler.app.overlay.agent_picker_index.saturating_sub(1);
            handler.app.mark_overlay_dirty();
        }
        KeyCode::Down => {
            let max = handler.app.overlay.agent_list.len().saturating_sub(1);
            if handler.app.overlay.agent_picker_index < max {
                handler.app.overlay.agent_picker_index += 1;
            }
            handler.app.mark_overlay_dirty();
        }
        KeyCode::Enter => {
            let agent_id = handler
                .app
                .overlay
                .agent_list
                .get(handler.app.overlay.agent_picker_index)
                .cloned();
            if let Some(ref agent_id) = agent_id
                && *agent_id != handler.app.current_agent
            {
                if let Some(old_id) = handler.app_core.session_mgr.current_id() {
                    let old_id_s = old_id.to_string();
                    crate::tui::clipboard::persist_session_messages(
                        &mut handler.app_core.session_mgr,
                        &old_id_s,
                        &handler.app.chat.messages,
                        handler.app.chat.api_messages.as_deref(),
                    );
                }
                handler.app.current_agent = agent_id.clone();
                handler.app.reset_for_new_session();
                handler.app.llm.status_text = format!("已切换到 agent: {}", agent_id);
                handler.app_core.session_mgr.create_session_for(agent_id, "default");
                handler
                    .app_core
                    .agent_store
                    .memory_for_mut("default", agent_id)
                    .expect("BUG: default agent runtime not initialized")
                    .analyze_sessions(
                        handler.app_core.session_mgr.sessions(),
                        &handler.app_core.session_mgr,
                    );
            }
            handler.app.overlay.close();
            handler.app.mark_overlay_dirty();
        }
        KeyCode::Esc => {
            handler.app.overlay.close();
            handler.app.mark_overlay_dirty();
        }
        _ => {}
    }
    Action::Continue
}

fn handle_theme_picker_keys(handler: &mut KeyEventHandler, key: KeyEvent) -> Action {
    let max = i_rs_claw_core::theme::BUILT_IN_THEMES.len().saturating_sub(1);
    match key.code {
        KeyCode::Up => {
            handler.app.overlay.theme_index = handler.app.overlay.theme_index.saturating_sub(1);
            apply_theme_preview(handler);
        }
        KeyCode::Down => {
            if handler.app.overlay.theme_index < max {
                handler.app.overlay.theme_index += 1;
            }
            apply_theme_preview(handler);
        }
        KeyCode::Enter => {
            apply_theme_preview(handler);
            save_theme(handler);
            handler.app.overlay.close();
        }
        KeyCode::Esc => {
            handler.app.overlay.close();
        }
        _ => {}
    }
    handler.app.mark_overlay_dirty();
    Action::Continue
}

fn apply_theme_preview(handler: &mut KeyEventHandler) {
    if let Some(preset) = i_rs_claw_core::theme::BUILT_IN_THEMES.get(handler.app.overlay.theme_index) {
        handler.app.config.theme =
            i_rs_claw_core::theme::Theme::from_preset(preset.name).unwrap_or_default();
    }
}

fn save_theme(handler: &KeyEventHandler) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let theme_path = home.join(".i-rs").join("claw").join("theme.json");
    if let Err(e) = handler.app.config.theme.save(&theme_path) {
        tracing::warn!("保存主题失败: {}", e);
    }
}

pub fn handle_export_session(handler: &mut KeyEventHandler) -> Action {
    if handler.app.chat.messages.is_empty() {
        handler.app.overlay.copy_feedback =
            Some(("无消息可导出".to_string(), std::time::Instant::now()));
        return Action::Continue;
    }

    let mut md = String::new();
    md.push_str(&format!(
        "# 会话导出 - {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    ));
    md.push_str(&format!("Agent: {}\n", handler.app.current_agent));
    md.push_str(&format!("模型: {}\n\n", handler.app.config.model));
    md.push_str("---\n\n");

    for msg in &handler.app.chat.messages {
        match msg {
            AppMessage::User { text } => {
                md.push_str("## 👤 用户\n\n");
                md.push_str(text);
                md.push_str("\n\n---\n\n");
            }
            AppMessage::Assistant { text, .. } => {
                md.push_str("## 🤖 Claw\n\n");
                md.push_str(text);
                md.push_str("\n\n---\n\n");
            }
            AppMessage::ToolCall {
                name, args, result, ..
            } => {
                md.push_str(&format!("## ⚡ 工具调用: `{}`\n\n", name));
                if let Ok(val) = serde_json::from_str::<Value>(args)
                    && name == "i_rs"
                {
                    let tool = val.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
                    let cmd = val.get("command").and_then(|v| v.as_str()).unwrap_or("?");
                    md.push_str(&format!("命令: `i-rs {} {}`\n\n", tool, cmd));
                }
                if !result.is_empty() {
                    let preview = if result.chars().count() > 500 {
                        let truncated: String = result.chars().take(500).collect();
                        format!("{}...", truncated)
                    } else {
                        result.clone()
                    };
                    md.push_str(&format!("```\n{}\n```\n\n", preview));
                }
                md.push_str("---\n\n");
            }
            AppMessage::Error { text } => {
                md.push_str("## ✗ 错误\n\n");
                md.push_str(&format!("```\n{}\n```\n\n", text));
                md.push_str("---\n\n");
            }
            AppMessage::Evaluation {
                tool,
                valid,
                issues,
            } => {
                if !valid {
                    md.push_str(&format!("## ⚠ 工具结果检查: `{}`\n\n", tool));
                    for issue in issues {
                        md.push_str(&format!("- {}\n", issue));
                    }
                    md.push_str("\n---\n\n");
                }
            }
            AppMessage::Quality {
                score,
                complete,
                issues,
                ..
            } => {
                md.push_str("## 📊 回答质量\n\n");
                if let Some(s) = score {
                    md.push_str(&format!("评分: {:.0}%\n", s * 100.0));
                }
                md.push_str(&format!(
                    "完整性: {}\n",
                    if *complete { "✅" } else { "❌" }
                ));
                for issue in issues {
                    md.push_str(&format!("- {}\n", issue));
                }
                md.push_str("\n---\n\n");
            }
            AppMessage::Feedback { positive, message } => {
                let icon = if *positive { "👍" } else { "👎" };
                md.push_str(&format!("## {} 用户反馈\n\n", icon));
                if let Some(msg) = message {
                    md.push_str(&format!("{}\n", msg));
                }
                md.push_str("\n---\n\n");
            }
            AppMessage::Image { path, alt_text, .. } => {
                md.push_str("## 🖼 图片\n\n");
                md.push_str(&format!("- 描述: {}\n", alt_text));
                md.push_str(&format!("- 路径: {}\n", path));
                md.push_str("\n---\n\n");
            }
        }
    }

    let export_dir = dirs::home_dir()
        .map(|h| h.join("Downloads"))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let filename = format!("claw-{}.md", chrono::Local::now().format("%Y%m%d-%H%M%S"));
    let path = export_dir.join(&filename);

    match std::fs::write(&path, &md) {
        Ok(_) => {
            handler.app.overlay.copy_feedback =
                Some((format!("✓ 已导出: {}", filename), std::time::Instant::now()));
        }
        Err(e) => {
            handler.app.overlay.copy_feedback =
                Some((format!("✗ 导出失败: {}", e), std::time::Instant::now()));
        }
    }
    handler.app.mark_overlay_dirty();
    Action::Continue
}

pub fn handle_slash_keys(handler: &mut KeyEventHandler, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            handler.app.overlay.slash_visible = false;
            handler.app.overlay.slash_index = 0;
            handler.app.input.text.clear();
            handler.app.input.cursor = 0;
            handler.app.mark_overlay_dirty();
            Action::Continue
        }
        KeyCode::Up => {
            handler.app.overlay.slash_index = handler.app.overlay.slash_index.saturating_sub(1);
            handler.app.mark_overlay_dirty();
            Action::Continue
        }
        KeyCode::Down => {
            let max = slash_match_count(handler.app).saturating_sub(1);
            if handler.app.overlay.slash_index < max {
                handler.app.overlay.slash_index += 1;
            }
            handler.app.mark_overlay_dirty();
            Action::Continue
        }
        KeyCode::Enter | KeyCode::Tab => handle_slash_execute(handler),
        KeyCode::Backspace => {
            handler.app.input.delete_before_cursor();
            if !handler.app.input.text.starts_with('/') {
                handler.app.overlay.slash_visible = false;
                handler.app.overlay.slash_index = 0;
            } else {
                handler.app.overlay.slash_index = 0;
            }
            handler.app.mark_overlay_dirty();
            Action::Continue
        }
        KeyCode::Char(c) => {
            handler.app.insert_char(c);
            handler.app.overlay.slash_index = 0;
            handler.app.mark_overlay_dirty();
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn slash_match_count(app: &App) -> usize {
    let query = if app.input.text.starts_with('/') {
        &app.input.text
    } else {
        ""
    };
    crate::app::SLASH_COMMANDS
        .iter()
        .filter(|cmd| {
            if query.is_empty() {
                return true;
            }
            let q = query.to_lowercase();
            cmd.name.starts_with(&q) || (query.len() > 1 && cmd.desc.contains(&query[1..]))
        })
        .count()
}

fn handle_slash_execute(handler: &mut KeyEventHandler) -> Action {
    let query = if handler.app.input.text.starts_with('/') {
        &handler.app.input.text
    } else {
        ""
    };
    let matches: Vec<&crate::app::SlashCommand> = crate::app::SLASH_COMMANDS
        .iter()
        .filter(|cmd| {
            if query.is_empty() {
                return true;
            }
            let q = query.to_lowercase();
            cmd.name.starts_with(&q) || (query.len() > 1 && cmd.desc.contains(&query[1..]))
        })
        .collect();

    let idx = handler
        .app
        .overlay
        .slash_index
        .min(matches.len().saturating_sub(1));
    let action = matches.get(idx).map(|cmd| cmd.action);

    handler.app.input.text.clear();
    handler.app.input.cursor = 0;
    handler.app.overlay.slash_visible = false;
    handler.app.overlay.slash_index = 0;

    match action {
        Some(crate::app::SlashAction::Help) => {
            handler.app.overlay.show(Overlay::Help);
        }
        Some(crate::app::SlashAction::Sessions) => {
            handler.app.overlay.show(Overlay::SessionList);
            handler.app.overlay.session_list_index = 0;
            handler.app.overlay.session_list = handler.app_core.session_mgr.sessions().to_vec();
        }
        Some(crate::app::SlashAction::New) => {
            return handler.handle_new_session();
        }
        Some(crate::app::SlashAction::Agent) => {
            handler.app.overlay.show(Overlay::AgentPicker);
            handler.app.overlay.agent_list = handler.app_core.config.agent_ids();
            handler.app.overlay.agent_picker_index = handler
                .app
                .overlay
                .agent_list
                .iter()
                .position(|id| *id == handler.app.current_agent)
                .unwrap_or(0);
        }
        Some(crate::app::SlashAction::Agents) => {
            handler.app.overlay.show(Overlay::AgentList);
        }
        Some(crate::app::SlashAction::Tools) => {
            handler.app.overlay.show(Overlay::ToolList);
        }
        Some(crate::app::SlashAction::Sidebar) => {
            handler.app.overlay.toggle(Overlay::Sidebar);
        }
        Some(crate::app::SlashAction::Stats) => {
            handler.app.overlay.show(Overlay::StatsHistory);
            handler.app.stats_history = handler.app_core.stats_manager.daily_history(7);
            handler.app.today_stats = handler.app_core.stats_manager.today_summary();
        }
        Some(crate::app::SlashAction::Plugins) => {
            handler.app.overlay.show(Overlay::PluginList);
            let store = handler
                .app_core
                .agent_store
                .skill_store_for("default", &handler.app.current_agent)
                .expect("BUG: default agent runtime not initialized");
            handler.app.skill_list = store.list_skills();
            let plugin_mgr = i_rs_claw_core::plugin::PluginManager::new();
            handler.app.plugin_list = plugin_mgr
                .manifests
                .iter()
                .map(|m| crate::app::PluginEntry {
                    name: m.plugin.name.clone(),
                    description: m.plugin.description.clone(),
                    enabled: plugin_mgr.is_enabled(&m.plugin.name),
                })
                .collect();
        }
        Some(crate::app::SlashAction::Config) => {
            handler.app.overlay.show(Overlay::Config);
        }
        Some(crate::app::SlashAction::Export) => {
            return handle_export_session(handler);
        }
        Some(crate::app::SlashAction::Feedback) => {
            handler.app.overlay.show(Overlay::Feedback);
        }
        Some(crate::app::SlashAction::Info) => {
            handler.app.overlay.show(Overlay::InfoPanel);
        }
        Some(crate::app::SlashAction::Select) => {
            if !handler.app.chat.messages.is_empty() {
                handler.app.overlay.selection_mode = true;
                handler.app.overlay.selected_message =
                    Some(handler.app.chat.messages.len().saturating_sub(1));
            }
        }
        Some(crate::app::SlashAction::Clear) => {
            handler.app.chat.messages.clear();
            handler.app.chat.message_timestamps.clear();
            handler.app.chat.components.clear();
            handler.app.chat.api_messages = None;
            handler.app.chat.tool_call_count = 0;
            handler.app.llm.status_text.clear();
            handler.app.scroll.scroll_lines = 0;
            handler.app.scroll.max_scroll = 0;
            handler.app.scroll.stick_to_bottom = true;
            handler.app.mark_dirty();
        }
        Some(crate::app::SlashAction::Compact) => {
            if let Some(_sid) = handler
                .app_core
                .session_mgr
                .current_id()
                .map(|s| s.to_string())
            {
                let memory = handler
                    .app_core
                    .agent_store
                    .memory_for_mut("default", &handler.app.current_agent)
                    .expect("BUG: default agent runtime not initialized");
                memory.analyze_sessions(
                    handler.app_core.session_mgr.sessions(),
                    &handler.app_core.session_mgr,
                );
                memory.flush();
                handler.app.overlay.copy_feedback =
                    Some(("✓ 记忆已更新（长期记忆提取完成）".to_string(), std::time::Instant::now()));
            }
        }
        Some(crate::app::SlashAction::Theme) => {
            handler.app.overlay.show(Overlay::ThemePicker);
            handler.app.overlay.theme_index = i_rs_claw_core::theme::BUILT_IN_THEMES
                .iter()
                .position(|t| {
                    let theme = &handler.app.config.theme;
                    theme.primary.as_deref() == Some(t.primary)
                        && theme.background.as_deref() == Some(t.background)
                })
                .unwrap_or(0);
        }
        None => {}
    }
    handler.app.mark_overlay_dirty();
    Action::Continue
}
