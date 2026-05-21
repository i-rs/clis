use crate::app;
use crate::core;
use crate::llm::{LlmEvent, TokenUsage};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use serde_json::Value;

use tokio::sync::mpsc;

/// Result of handling a key event.
pub enum Action {
    Continue,
    Quit,
}

// ─── LlmEventHandler ────────────────────────────────────────────

pub struct LlmEventHandler<'a> {
    pub app: &'a mut app::App,
    pub app_core: &'a mut core::AppCore,
}

impl<'a> LlmEventHandler<'a> {
    pub fn new(app: &'a mut app::App, app_core: &'a mut core::AppCore) -> Self {
        Self { app, app_core }
    }

    /// Handle a single LlmEvent. Returns `Action::Quit` when the Done
    /// handler encounters a fatal session error.
    pub fn handle(&mut self, event: LlmEvent) -> Action {
        match event {
            LlmEvent::NewRound => self.handle_new_round(),
            LlmEvent::Token(text) => self.handle_token(&text),
            LlmEvent::Reasoning(text) => self.handle_reasoning(&text),
            LlmEvent::Status(text) => self.handle_status(&text),
            LlmEvent::ToolExecuted { name, args, result, step, total_steps } => {
                self.handle_tool_executed(&name, &args, &result, step, total_steps);
            }
            LlmEvent::Error(text) => self.handle_error(&text),
            LlmEvent::HttpLog(data) => {
                self.handle_http_log(&data);
            }
            LlmEvent::UsageRecord(record) => {
                self.app_core.stats_manager.record(record);
                self.app.today_stats = self.app_core.stats_manager.today_summary();
            }
            LlmEvent::Done(msgs, usage) => return self.handle_done((*msgs).clone(), usage),
        }
        Action::Continue
    }

    fn handle_new_round(&mut self) {
        self.app.start_assistant_message();
    }

    fn handle_token(&mut self, text: &str) {
        self.app.append_assistant_text(text);
        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute
            && let Some(crate::app::Message::Assistant { text: t }) = self.app.messages.last() {
                let plan_text = t.clone();
                if !plan_text.is_empty() {
                    self.app.detect_plan(&plan_text);
                    if let Some(sid) = self.app_core.session_mgr.current_id() {
                        self.app_core.session_mgr.save_plan_steps(sid, &self.app.plan_steps);
                    }
                }
            }
    }

    fn handle_reasoning(&mut self, text: &str) {
        self.app.current_reasoning.push_str(text);
    }

    fn handle_status(&mut self, text: &str) {
        self.app.set_status(text);
        if (text.starts_with("⚡") || text.contains("并行执行"))
            && let Some(sid) = self.app_core.session_mgr.current_id().map(|s| s.to_string())
        {
            self.app_core.session_mgr.mark_waiting_for_tool(&sid);
        }
    }

    fn handle_tool_executed(&mut self, name: &str, args: &str, result: &str, step: usize, total_steps: usize) {
        self.app.add_tool_call(name, args, result, step, total_steps);

        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute {
            self.app.mark_next_plan_step_done();
            if let Some(sid) = self.app_core.session_mgr.current_id() {
                self.app_core.session_mgr.save_plan_steps(sid, &self.app.plan_steps);
            }
        }

        // Save user information from update_user_memory tool
        if name == "update_user_memory"
            && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(args)
        {
            if let Some(user_name) = parsed.get("user_name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                self.app_core.agent_store.memory_for_mut(&self.app.current_agent).set_user_name(user_name);
            }
            if let Some(info) = parsed.get("user_info").and_then(|v| v.as_array()) {
                for item in info {
                    if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                        self.app_core.agent_store.memory_for_mut(&self.app.current_agent).add_user_info(s);
                    }
                }
            }
            if let Some(prefs) = parsed.get("preferences").and_then(|v| v.as_array()) {
                for item in prefs {
                    if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                        self.app_core.agent_store.memory_for_mut(&self.app.current_agent).add_preference(s);
                    }
                }
            }
        }

        // Record tool usage for cross-session memory
        let agent_id = self.app.current_agent.clone();
        if name == "i_rs" {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(args)
                && let Some(tool) = parsed.get("tool").and_then(|t| t.as_str())
            {
                self.app_core.agent_store.memory_for_mut(&agent_id).record_tool_use(tool);
            }
        } else {
            self.app_core.agent_store.memory_for_mut(&agent_id).record_tool_use(name);
        }
    }

    fn handle_error(&mut self, text: &str) {
        self.app.add_error(text);
        if let Some(sid) = self.app_core.session_mgr.current_id().map(|s| s.to_string()) {
            self.app_core.session_mgr.mark_error(&sid, text);
        }
    }

    fn handle_http_log(&mut self, data: &crate::llm::HttpLogData) {
        self.app.add_http_log(app::HttpLog {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            status: data.status,
            duration_ms: data.duration_ms,
            model: data.model.clone(),
            prompt_tokens: data.prompt_tokens,
            completion_tokens: data.completion_tokens,
            error: data.error.clone(),
            request_body: data.request_body.clone(),
        });
    }

    fn handle_done(&mut self, msgs: Vec<Value>, usage: Option<TokenUsage>) -> Action {
        let mut msgs = msgs.clone();
        self.app_core.compress_api_messages(&mut msgs, &self.app.current_agent);

        self.app.finish_processing(Some(msgs.clone()));
        self.app.token_usage = usage;

        if let Some(sid) = self.app_core.session_mgr.current_id().map(|s| s.to_string()) {
            self.app_core.session_mgr.mark_active(&sid);
        }

        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute
            && let Some(sid) = self.app_core.session_mgr.current_id()
        {
            self.app_core.session_mgr.save_plan_steps(sid, &[]);
        }

        let session_id = match self.app_core.session_mgr.current_id() {
            Some(id) => id.to_string(),
            None => {
                tracing::warn!("未找到当前会话，跳过持久化");
                self.app.finish_processing(None);
                return Action::Quit;
            }
        };

        // Persist
        crate::tui::clipboard::save_session_messages(
            &self.app_core.session_mgr,
            &session_id,
            &self.app.messages,
            Some(&msgs),
        );

        // Rename session based on first user message
        let needs_rename = self.app_core.session_mgr
            .current_session()
            .map(|s| s.title == "新对话" || s.title.is_empty())
            .unwrap_or(false);
        if needs_rename
            && let Some(first_user) = self.app.messages.iter().find_map(|m| {
                if let crate::app::Message::User { text } = m {
                    Some(text.clone())
                } else {
                    None
                }
            })
        {
            self.app_core.session_mgr.rename_session(&session_id, &first_user);
        }

        // Flush pending memory writes
        self.app_core.agent_store.memory_for_mut(&self.app.current_agent).flush();

        Action::Continue
    }
}

// ─── KeyEventHandler ────────────────────────────────────────────

pub struct KeyEventHandler<'a> {
    pub app: &'a mut app::App,
    pub app_core: &'a mut core::AppCore,
    pub rt: &'a tokio::runtime::Runtime,
    pub llm_tx: &'a mpsc::UnboundedSender<LlmEvent>,
}

impl<'a> KeyEventHandler<'a> {
    pub fn new(
        app: &'a mut app::App,
        app_core: &'a mut core::AppCore,
        rt: &'a tokio::runtime::Runtime,
        llm_tx: &'a mpsc::UnboundedSender<LlmEvent>,
    ) -> Self {
        Self { app, app_core, rt, llm_tx }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        match key.code {
            // ── Global shortcuts (always active) ────────────────
            KeyCode::Char('c') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                return self.handle_copy();
            }
            KeyCode::Char('q') | KeyCode::Char('c')
                if key.modifiers == KeyModifiers::CONTROL =>
            {
                return Action::Quit;
            }

            // ── Overlay-toast dismissals (high priority) ─────────
            KeyCode::Esc if self.app.overlay.show_config => {
                self.app.overlay.show_config = false;
            }
            KeyCode::Esc | KeyCode::Enter if self.app.overlay.show_help => {
                self.app.overlay.show_help = false;
            }
            KeyCode::Esc if self.app.overlay.selection_mode => {
                self.app.overlay.selection_mode = false;
                self.app.overlay.selected_message = None;
            }
            KeyCode::Char('q') if self.app.overlay.selection_mode => {
                self.app.overlay.selection_mode = false;
                self.app.overlay.selected_message = None;
            }

            // ── Selection mode ───────────────────────────────────
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL
                && !self.app.messages.is_empty() => {
                    self.app.overlay.selection_mode = !self.app.overlay.selection_mode;
                    self.app.overlay.selected_message = if self.app.overlay.selection_mode {
                        Some(self.app.messages.len().saturating_sub(1))
                    } else {
                        None
                    };
                }
            KeyCode::Char('d')
                if key.modifiers == KeyModifiers::CONTROL && self.app.overlay.selection_mode =>
            {
                return self.handle_delete_selected_message();
            }
            KeyCode::Up if self.app.overlay.selection_mode => {
                if let Some(idx) = self.app.overlay.selected_message
                    && idx > 0 {
                        self.app.overlay.selected_message = Some(idx - 1);
                    }
            }
            KeyCode::Down if self.app.overlay.selection_mode => {
                if let Some(idx) = self.app.overlay.selected_message
                    && idx + 1 < self.app.messages.len() {
                        self.app.overlay.selected_message = Some(idx + 1);
                        let bottom_is_newer_rev = self.app.messages.len().saturating_sub(1) - (idx + 1);
                        if bottom_is_newer_rev > 0 && self.app.scroll_lines > 0 {
                            self.app.scroll_lines = 0;
                        }
                    }
            }
            KeyCode::Char(' ') if self.app.overlay.selection_mode => {
                if let Some(idx) = self.app.overlay.selected_message
                    && matches!(self.app.messages.get(idx), Some(crate::app::Message::ToolCall { .. }))
                        && !self.app.overlay.tool_call_expanded.remove(&idx) {
                            self.app.overlay.tool_call_expanded.insert(idx);
                        }
            }

            // ── Help shortcut ────────────────────────────────────
            KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                self.app.overlay.show_help = !self.app.overlay.show_help;
                if self.app.overlay.show_help {
                    self.app.overlay.show_config = false;
                }
            }

            // ── Config info shortcut ─────────────────────────────
            KeyCode::Char('i') if key.modifiers == KeyModifiers::CONTROL => {
                self.app.overlay.show_config = !self.app.overlay.show_config;
                if self.app.overlay.show_config {
                    self.app.overlay.show_help = false;
                }
            }

            // ── Session list overlay ─────────────────────────────
            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                self.app.overlay.show_session_list = !self.app.overlay.show_session_list;
                if self.app.overlay.show_session_list {
                    self.app.overlay.session_list_index = 0;
                    self.app.overlay.session_list = self.app_core.session_mgr.sessions().to_vec();
                }
            }
            KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                return self.handle_new_session();
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                return self.handle_sidebar_toggle();
            }
            KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                return self.handle_agent_picker_toggle();
            }

            // ── Session list keys ────────────────────────────────
            _ if self.app.overlay.show_session_list => {
                return self.handle_session_list_keys(key);
            }

            // ── Sidebar keys ─────────────────────────────────────
            _ if self.app.overlay.show_sidebar => {
                return self.handle_sidebar_keys(key);
            }

            // ── Agent picker keys ────────────────────────────────
            _ if self.app.overlay.show_agent_picker => {
                return self.handle_agent_picker_keys(key);
            }

            // ── Tab completions ──────────────────────────────────
            KeyCode::Esc if !self.app.overlay.tab_completions.is_empty() => {
                self.app.overlay.tab_completions.clear();
                self.app.overlay.tab_completion_index = 0;
            }
            KeyCode::Tab if !self.app.is_processing() && !self.app.input.text.is_empty() => {
                self.handle_tab_complete();
            }
            KeyCode::BackTab if !self.app.is_processing() && !self.app.overlay.tab_completions.is_empty() => {
                self.handle_backtab_complete();
            }

            // ── Normal mode (input editing + navigation) ─────────
            _ => {
                return self.handle_normal_input(key);
            }
        }
        Action::Continue
    }

    // ── Copy ────────────────────────────────────────────────

    fn handle_copy(&mut self) -> Action {
        let content = if self.app.overlay.selection_mode {
            self.app.overlay.selected_message.and_then(|idx| {
                self.app.messages.get(idx).map(|m| match m {
                    crate::app::Message::User { text } => text.clone(),
                    crate::app::Message::Assistant { text } => text.clone(),
                    crate::app::Message::ToolCall { name, args, result, .. } =>
                        format!("Tool: {}\nArgs: {}\nResult: {}", name, args, result),
                    crate::app::Message::Error { text } => text.clone(),
                })
            })
        } else {
            self.app.messages
                .iter()
                .rev()
                .find_map(|m| match m {
                    crate::app::Message::Assistant { text } if !text.is_empty() => Some(text.clone()),
                    _ => None,
                })
        };
        if let Some(content) = content {
            if crate::tui::clipboard::copy_to_clipboard(&content) {
                self.app.overlay.copy_feedback = Some("✓ 已复制".to_string());
            } else {
                self.app.overlay.copy_feedback = Some("✗ 复制失败".to_string());
            }
        } else {
            self.app.overlay.copy_feedback = Some("无内容可复制".to_string());
        }
        Action::Continue
    }

    fn handle_delete_selected_message(&mut self) -> Action {
        if let Some(idx) = self.app.overlay.selected_message {
            let idx = idx.min(self.app.messages.len().saturating_sub(1));
            self.app.messages.remove(idx);
            self.app.message_timestamps.remove(idx);
            self.app.overlay.tool_call_expanded.remove(&idx);
            let tc = std::mem::take(&mut self.app.overlay.tool_call_expanded);
            self.app.overlay.tool_call_expanded = tc.into_iter().map(|i| if i > idx { i - 1 } else { i }).collect();
            if idx >= self.app.messages.len() {
                self.app.overlay.selected_message = if self.app.messages.is_empty() { None } else { Some(self.app.messages.len() - 1) };
            }
        }
        Action::Continue
    }

    fn handle_new_session(&mut self) -> Action {
        if let Some(old_id) = self.app_core.session_mgr.current_id().map(|id| id.to_string()) {
            crate::tui::clipboard::save_session_messages(
                &self.app_core.session_mgr,
                &old_id,
                &self.app.messages,
                self.app.api_messages.as_deref(),
            );
        }
        self.app_core.session_mgr.create_session();
        self.app.reset_for_new_session();
        self.app.overlay.show_session_list = false;
        Action::Continue
    }

    // ── Session list sub-handler ─────────────────────────────

    fn handle_session_list_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('d')
                if key.modifiers == KeyModifiers::CONTROL => {
                    self.app.overlay.session_confirm_delete = true;
                }
            KeyCode::Char('r')
                if key.modifiers == KeyModifiers::CONTROL => {
                    let filtered = self.app.overlay.filtered_sessions();
                    if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
                        self.app.overlay.session_rename_buf = meta.title.clone();
                    }
                }
            KeyCode::Char('y') if self.app.overlay.session_confirm_delete => {
                let filtered = self.app.overlay.filtered_sessions();
                if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
                    let id = meta.id.clone();
                    let is_current = self.app_core.session_mgr
                        .current_id()
                        .map(|cid| cid == id)
                        .unwrap_or(false);
                    self.app_core.session_mgr.delete_session(&id);
                    self.app.overlay.session_confirm_delete = false;
                    self.app.overlay.session_list = self.app_core.session_mgr.sessions().to_vec();
                    if is_current {
                        self.app.reset_for_new_session();
                    }
                } else {
                    self.app.overlay.session_confirm_delete = false;
                }
            }
            KeyCode::Char('n') if self.app.overlay.session_confirm_delete => {
                self.app.overlay.session_confirm_delete = false;
            }
            KeyCode::Esc => {
                if self.app.overlay.session_search_mode {
                    self.app.overlay.session_search_mode = false;
                    self.app.overlay.session_search.clear();
                } else if self.app.overlay.session_confirm_delete {
                    self.app.overlay.session_confirm_delete = false;
                } else if !self.app.overlay.session_rename_buf.is_empty() {
                    self.app.overlay.session_rename_buf.clear();
                } else {
                    self.app.overlay.show_session_list = false;
                }
            }
            KeyCode::Up => {
                self.app.overlay.session_list_index =
                    self.app.overlay.session_list_index.saturating_sub(1);
            }
            KeyCode::Down if !self.app.overlay.session_search_mode => {
                let max = self.app.overlay.session_list.len().saturating_sub(1);
                if self.app.overlay.session_list_index < max {
                    self.app.overlay.session_list_index += 1;
                }
            }
            KeyCode::Char('/') if !self.app.overlay.session_search_mode => {
                self.app.overlay.session_search_mode = true;
                self.app.overlay.session_search.clear();
            }
            KeyCode::Char(c) if self.app.overlay.session_search_mode => {
                self.app.overlay.session_search.push(c);
                self.app.overlay.session_list_index = 0;
            }
            KeyCode::Backspace if self.app.overlay.session_search_mode => {
                self.app.overlay.session_search.pop();
                self.app.overlay.session_list_index = 0;
            }
            KeyCode::Enter => {
                return self.handle_session_enter();
            }
            _ => {}
        }
        Action::Continue
    }

    fn handle_session_enter(&mut self) -> Action {
        // If renaming, confirm rename
        if !self.app.overlay.session_rename_buf.is_empty() {
            let filtered = self.app.overlay.filtered_sessions();
            if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
                let title = std::mem::take(&mut self.app.overlay.session_rename_buf);
                if !title.trim().is_empty() {
                    self.app_core.session_mgr.rename_session(&meta.id, title.trim());
                }
                self.app.overlay.session_list = self.app_core.session_mgr.sessions().to_vec();
            } else {
                self.app.overlay.session_rename_buf.clear();
            }
            // NOTE: 原代码这里 break 会退出整个 TUI（原有 bug），改为 Continue
            return Action::Continue;
        }

        // Switch to selected session
        let filtered = self.app.overlay.filtered_sessions();

        if self.app.overlay.session_search_mode {
            self.app.overlay.session_search_mode = false;
        }

        if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
            let new_id = meta.id.clone();
            let is_current = self.app_core.session_mgr
                .current_id()
                .map(|id| id == new_id)
                .unwrap_or(false);
            if !is_current {
                let old_id = self.app_core.session_mgr
                    .current_id()
                    .unwrap_or_default()
                    .to_string();
                crate::tui::clipboard::save_session_messages(
                    &self.app_core.session_mgr,
                    &old_id,
                    &self.app.messages,
                    self.app.api_messages.as_deref(),
                );

                self.app_core.session_mgr.switch_to(&new_id);
                let loaded = self.app_core.session_mgr.load_app_messages(&new_id, 50);
                self.app.messages = loaded;
                self.app.sync_message_timestamps();
                self.app.api_messages = self.app_core.session_mgr.load_api_messages(&new_id);
                self.app.tool_call_count = 0;
                self.app.status_text.clear();
                self.app.token_usage = None;
                self.app.plan_steps = self.app_core.session_mgr.load_plan_steps(&new_id);
            }
        }
        self.app.overlay.show_session_list = false;
        Action::Continue
    }

    // ── Sidebar sub-handler ──────────────────────────────────

    fn handle_sidebar_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc if self.app.overlay.sidebar_body_idx.is_some() => {
                self.app.overlay.sidebar_body_idx = None;
            }
            KeyCode::Esc => {
                self.app.overlay.show_sidebar = false;
            }
            KeyCode::Up if !self.app.overlay.show_session_list && self.app.overlay.sidebar_body_idx.is_none() => {
                self.app.overlay.sidebar_selected = self.app.overlay.sidebar_selected.saturating_sub(1);
            }
            KeyCode::Down if !self.app.overlay.show_session_list && self.app.overlay.sidebar_body_idx.is_none() => {
                let max = self.app.http_logs.len().saturating_sub(1);
                if self.app.overlay.sidebar_selected < max {
                    self.app.overlay.sidebar_selected += 1;
                }
            }
            KeyCode::Enter if !self.app.overlay.show_session_list
                && self.app.overlay.sidebar_body_idx.is_none()
                && !self.app.http_logs.is_empty() => {
                    self.app.overlay.sidebar_body_idx = Some(self.app.overlay.sidebar_selected);
                }
            KeyCode::Up if !self.app.overlay.show_session_list && self.app.overlay.sidebar_body_idx.is_some() => {
                self.app.overlay.sidebar_body_scroll = self.app.overlay.sidebar_body_scroll.saturating_sub(1);
            }
            KeyCode::Down if !self.app.overlay.show_session_list && self.app.overlay.sidebar_body_idx.is_some() => {
                self.app.overlay.sidebar_body_scroll += 1;
            }
            _ => {}
        }
        Action::Continue
    }

    // ── Agent picker sub-handler ─────────────────────────────

    fn handle_agent_picker_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up => {
                self.app.overlay.agent_picker_index = self.app.overlay.agent_picker_index.saturating_sub(1);
            }
            KeyCode::Down => {
                let max = self.app.overlay.agent_list.len().saturating_sub(1);
                if self.app.overlay.agent_picker_index < max {
                    self.app.overlay.agent_picker_index += 1;
                }
            }
            KeyCode::Enter => {
                let agent_id = self.app.overlay.agent_list.get(self.app.overlay.agent_picker_index).cloned();
                if let Some(ref agent_id) = agent_id
                    && *agent_id != self.app.current_agent {
                        // Save current session messages
                        if let Some(old_id) = self.app_core.session_mgr.current_id().map(|id| id.to_string()) {
                            crate::tui::clipboard::save_session_messages(
                                &self.app_core.session_mgr,
                                &old_id,
                                &self.app.messages,
                                self.app.api_messages.as_deref(),
                            );
                        }
                        self.app.current_agent = agent_id.clone();
                        self.app.reset_for_new_session();
                        self.app.status_text = format!("已切换到 agent: {}", agent_id);
                        self.app_core.session_mgr.create_session_for(agent_id);
                        self.app_core.agent_store.memory_for_mut(agent_id)
                            .analyze_sessions(self.app_core.session_mgr.sessions(), &self.app_core.session_mgr);
                    }
                self.app.overlay.show_agent_picker = false;
            }
            KeyCode::Esc => {
                self.app.overlay.show_agent_picker = false;
            }
            _ => {}
        }
        Action::Continue
    }

    // ── Sidebar shortcuts (Ctrl+R, Ctrl+S, Ctrl+P) ──────────



    fn handle_sidebar_toggle(&mut self) -> Action {
        // Ctrl+R: toggle HTTP debug sidebar
        self.app.overlay.show_sidebar = !self.app.overlay.show_sidebar;
        if self.app.overlay.show_sidebar {
            self.app.overlay.show_session_list = false;
        }
        Action::Continue
    }

    fn handle_agent_picker_toggle(&mut self) -> Action {
        // Ctrl+P: toggle agent picker
        self.app.overlay.show_agent_picker = !self.app.overlay.show_agent_picker;
        if self.app.overlay.show_agent_picker {
            self.app.overlay.show_session_list = false;
            self.app.overlay.show_sidebar = false;
            self.app.overlay.sidebar_body_idx = None;
            self.app.overlay.agent_list = self.app_core.config.agent_ids();
            self.app.overlay.agent_picker_index = self.app.overlay.agent_list
                .iter()
                .position(|id| *id == self.app.current_agent)
                .unwrap_or(0);
        }
        Action::Continue
    }

    // ── Normal input / navigation ───────────────────────────

    fn handle_normal_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up if !self.app.overlay.selection_mode
                && !self.app.overlay.show_sidebar
                && !self.app.is_processing() => {
                    if self.app.input.text.is_empty() {
                        self.app.scroll_up();
                    } else if let Some(text) = self.app.input.navigate_up() {
                        self.app.input.text = text;
                        self.app.input.move_cursor_end();
                    }
                }
            KeyCode::Down if !self.app.overlay.selection_mode
                && !self.app.overlay.show_sidebar
                && !self.app.is_processing() => {
                    if self.app.input.text.is_empty() {
                        self.app.scroll_down();
                    } else if let Some(text) = self.app.input.navigate_down() {
                        self.app.input.text = text;
                        self.app.input.move_cursor_end();
                    } else {
                        self.app.input.text.clear();
                        self.app.input.cursor = 0;
                    }
                }
            KeyCode::Enter => {
                if key.modifiers == KeyModifiers::ALT {
                    if !self.app.is_processing() {
                        self.app.insert_char('\n');
                    }
                } else if !self.app.input.text.is_empty() && !self.app.is_processing() {
                    let text = std::mem::take(&mut self.app.input.text);
                    self.app.input.cursor = 0;
                    self.app.commit_input_to_history(&text);
                    self.app.add_user_message(&text);
                    self.app_core.session_mgr.append_message("user", &text, None);
                    let msgs = self.app_core.build_messages_for(
                        &self.app.messages,
                        &text,
                        &self.app.api_messages,
                        self.app.reminder_text.as_deref(),
                        &self.app.current_agent,
                    );
                    self.app_core.spawn_chat_for(self.rt, self.llm_tx.clone(), msgs, &self.app.current_agent);
                }
            }
            KeyCode::Backspace if !self.app.input.text.is_empty() => {
                self.app.delete_before_cursor();
                if !self.app.overlay.tab_completions.is_empty() {
                    self.app.overlay.tab_completions.clear();
                    self.app.overlay.tab_completion_index = 0;
                }
            }
            KeyCode::Left => { self.app.move_cursor_left(); }
            KeyCode::Right => { self.app.move_cursor_right(); }
            KeyCode::Home => { self.app.input.move_cursor_home(); }
            KeyCode::End => { self.app.input.move_cursor_end(); }
            KeyCode::Char(c) if !self.app.is_processing() => {
                if !self.app.overlay.tab_completions.is_empty() {
                    self.app.overlay.tab_completions.clear();
                    self.app.overlay.tab_completion_index = 0;
                }
                self.app.insert_char(c);
            }
            _ => {}
        }
        Action::Continue
    }

    // ── Tab completion ──────────────────────────────────────

    fn handle_tab_complete(&mut self) {
        let completions = crate::completion::get_completions(&self.app.input.text, self.app.input.cursor);
        if !completions.is_empty() {
            if self.app.overlay.tab_completions.is_empty() {
                self.app.overlay.tab_completions = completions;
                self.app.overlay.tab_completion_index = 0;
            } else {
                self.app.overlay.tab_completion_index = (self.app.overlay.tab_completion_index + 1) % self.app.overlay.tab_completions.len();
            }
            let selected = &self.app.overlay.tab_completions[self.app.overlay.tab_completion_index];
            let before = &self.app.input.text[..self.app.input.cursor];
            let after = &self.app.input.text[self.app.input.cursor..];
            let word_start = before.rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
            self.app.input.text = format!("{}{} {}", &before[..word_start], selected, after);
            self.app.input.cursor = word_start + selected.len() + 1;
        }
    }

    fn handle_backtab_complete(&mut self) {
        let len = self.app.overlay.tab_completions.len();
        self.app.overlay.tab_completion_index = if self.app.overlay.tab_completion_index == 0 {
            len.saturating_sub(1)
        } else {
            self.app.overlay.tab_completion_index - 1
        };
        let selected = &self.app.overlay.tab_completions[self.app.overlay.tab_completion_index];
        let before = &self.app.input.text[..self.app.input.cursor];
        let after = &self.app.input.text[self.app.input.cursor..];
        let word_start = before.rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
        self.app.input.text = format!("{}{} {}", &before[..word_start], selected, after);
        self.app.input.cursor = word_start + selected.len() + 1;
    }
}

// ─── MouseEventHandler ──────────────────────────────────────────

pub struct MouseEventHandler<'a> {
    pub app: &'a mut app::App,
}

impl<'a> MouseEventHandler<'a> {
    pub fn new(app: &'a mut app::App) -> Self {
        Self { app }
    }

    pub fn handle(&mut self, mouse: MouseEvent) {
        if self.app.overlay.sidebar_body_idx.is_some() {
            match mouse.kind {
                MouseEventKind::ScrollDown => {
                    self.app.overlay.sidebar_body_scroll += 1;
                }
                MouseEventKind::ScrollUp => {
                    self.app.overlay.sidebar_body_scroll = self.app
                        .overlay.sidebar_body_scroll
                        .saturating_sub(1);
                }
                _ => {}
            }
        } else if !self.app.is_processing() && !self.app.overlay.show_sidebar {
            match mouse.kind {
                MouseEventKind::ScrollDown => self.app.scroll_up(),
                MouseEventKind::ScrollUp => self.app.scroll_down(),
                _ => {}
            }
        }
    }
}
