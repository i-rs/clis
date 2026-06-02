use crate::app::{self, App, Overlay};
use crate::core;
use crate::llm::{LlmEvent, TokenUsage};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use serde_json::Value;

use tokio::sync::mpsc;

pub enum Action {
    Continue,
    Quit,
}

pub struct LlmEventHandler<'a> {
    pub app: &'a mut App,
    pub app_core: &'a mut core::AppCore,
}

impl<'a> LlmEventHandler<'a> {
    pub fn new(app: &'a mut App, app_core: &'a mut core::AppCore) -> Self {
        Self { app, app_core }
    }

    pub fn handle(&mut self, event: LlmEvent) -> Action {
        match event {
            LlmEvent::NewRound => self.handle_new_round(),
            LlmEvent::Token(text) => self.handle_token(&text),
            LlmEvent::Reasoning(text) => self.handle_reasoning(&text),
            LlmEvent::Status(text) => self.handle_status(&text),
            LlmEvent::ToolExecuted {
                name,
                args,
                result,
                step,
                total_steps,
            } => {
                self.handle_tool_executed(&name, &args, &result, step, total_steps);
            }
            LlmEvent::Error(text) => self.handle_error(&text),
            LlmEvent::HttpLog(data) => {
                self.handle_http_log(&data);
            }
            LlmEvent::UsageRecord(record) => {
                self.app_core.stats_manager.record(record);
                self.app.today_stats = self.app_core.stats_manager.today_summary();
                self.app.mark_dirty();
            }
            LlmEvent::Done(msgs, usage, _trace_id) => {
                return self.handle_done((*msgs).clone(), usage);
            }
            LlmEvent::Evaluation {
                tool,
                valid,
                issues,
            } => {
                self.handle_evaluation(&tool, valid, &issues);
            }
            LlmEvent::PlanProgress(steps) => {
                self.app.plan_steps = steps;
                self.app.mark_dirty();
            }
            LlmEvent::ImageGenerated { path, alt_text, format: _, width, height } => {
                self.app.messages.push(app::Message::Image {
                    path,
                    alt_text,
                    width,
                    height,
                    format: "png".to_string(),
                });
                self.app.message_timestamps.push(chrono::Local::now().naive_local());
                self.app.mark_dirty();
            }
        }
        Action::Continue
    }

    fn handle_new_round(&mut self) {
        self.app.start_assistant_message();
    }

    fn handle_token(&mut self, text: &str) {
        self.app.append_assistant_text(text);
        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute
            && (text.contains('\n') || self.app.plan_steps.is_empty())
            && let Some(AppMessage::Assistant { text: t, .. }) = self.app.messages.last()
        {
            let plan_text = t.clone();
            if !plan_text.is_empty() {
                self.app.detect_plan(&plan_text);
                if let Some(sid) = self.app_core.session_mgr.current_id() {
                    self.app_core
                        .session_mgr
                        .save_plan_steps(sid, &self.app.plan_steps);
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
            && let Some(sid) = self
                .app_core
                .session_mgr
                .current_id()
                .map(|s| s.to_string())
        {
            self.app_core.session_mgr.mark_waiting_for_tool(&sid);
        }
    }

    fn handle_tool_executed(
        &mut self,
        name: &str,
        args: &str,
        result: &str,
        step: usize,
        total_steps: usize,
    ) {
        self.app
            .add_tool_call(name, args, result, step, total_steps);

        if self.app.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute {
            self.app.mark_next_plan_step_done();
            if let Some(sid) = self.app_core.session_mgr.current_id() {
                self.app_core
                    .session_mgr
                    .save_plan_steps(sid, &self.app.plan_steps);
            }
        }

        let agent_id = self.app.current_agent.clone();
        let i_rs_index = self.app_core.config.i_rs_tool_index.clone();
        crate::core::record_tool_memory(
            &mut self.app_core.agent_store,
            &i_rs_index,
            &agent_id,
            name,
            args,
            result,
        );
        if !result.starts_with("错误") && !result.starts_with("护栏拦截") {
            crate::core::record_layered_tool_memory(
                &mut self.app_core.agent_store,
                &agent_id,
                name,
                result,
            );
        }
    }

    fn handle_error(&mut self, text: &str) {
        self.app.add_error(text);
        if let Some(sid) = self
            .app_core
            .session_mgr
            .current_id()
            .map(|s| s.to_string())
        {
            self.app_core.session_mgr.mark_error(&sid, text);
        }
    }

    fn handle_evaluation(&mut self, tool: &str, valid: bool, issues: &[String]) {
        self.app.messages.push(app::Message::Evaluation {
            tool: tool.to_string(),
            valid,
            issues: issues.to_vec(),
        });
        self.app.message_timestamps.push(chrono::Local::now().naive_local());
        self.app.mark_dirty();
        if !valid {
            tracing::info!(tool, issues = ?issues, "工具结果验证告警");
        }
    }

    fn handle_http_log(&mut self, data: &crate::llm::HttpLogData) {
        let msg_count = serde_json::from_str::<serde_json::Value>(&data.request_body)
            .ok()
            .and_then(|v| v["messages"].as_array().map(|a| a.len()))
            .unwrap_or(0);
        self.app.add_http_log(app::HttpLog {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            status: data.status,
            duration_ms: data.duration_ms,
            model: data.model.clone(),
            prompt_tokens: data.prompt_tokens,
            completion_tokens: data.completion_tokens,
            error: data.error.clone(),
            request_body: data.request_body.clone(),
            msg_count,
        });
    }

    fn handle_done(&mut self, msgs: Vec<Value>, usage: Option<TokenUsage>) -> Action {
        let mut msgs = msgs;
        self.app_core
            .compress_api_messages(&mut msgs, &self.app.current_agent);

        self.app.finish_processing(Some(msgs.clone()));
        self.app.token_usage = usage;

        if let Some(sid) = self
            .app_core
            .session_mgr
            .current_id()
            .map(|s| s.to_string())
        {
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
                return Action::Continue;
            }
        };

        crate::tui::clipboard::save_session_messages(
            &self.app_core.session_mgr,
            &session_id,
            &self.app.messages,
            Some(&msgs),
        );

        let needs_rename = self
            .app_core
            .session_mgr
            .current_session()
            .map(|s| s.title == "新对话" || s.title.is_empty())
            .unwrap_or(false);
        if needs_rename
            && let Some(first_user) = self.app.messages.iter().find_map(|m| {
                if let app::Message::User { text } = m {
                    Some(text.clone())
                } else {
                    None
                }
            })
        {
            self.app_core
                .session_mgr
                .rename_session(&session_id, &first_user);
        }

        self.app_core
            .agent_store
            .memory_for_mut(&self.app.current_agent)
            .flush();

        if let Some(quality) = self.app_core.evaluate_completed_session(&session_id) {
            self.app.messages.push(quality);
            self.app.message_timestamps.push(chrono::Local::now().naive_local());
            self.app.mark_dirty();
        }

        Action::Continue
    }
}

pub struct KeyEventHandler<'a> {
    pub app: &'a mut App,
    pub app_core: &'a mut core::AppCore,
    pub rt: &'a tokio::runtime::Runtime,
    pub llm_tx: &'a mpsc::UnboundedSender<LlmEvent>,
}

impl<'a> KeyEventHandler<'a> {
    pub fn new(
        app: &'a mut App,
        app_core: &'a mut core::AppCore,
        rt: &'a tokio::runtime::Runtime,
        llm_tx: &'a mpsc::UnboundedSender<LlmEvent>,
    ) -> Self {
        Self {
            app,
            app_core,
            rt,
            llm_tx,
        }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        if self.handle_global_shortcuts(key) {
            return self.handle_global_action(key);
        }
        if self.handle_overlay_dismissals(key) {
            return Action::Continue;
        }
        if self.handle_selection_mode(key) {
            return Action::Continue;
        }
        if self.handle_overlay_shortcuts(key) {
            return self.handle_overlay_action(key);
        }
        if let Some(overlay) = self.app.overlay.current {
            return self.handle_active_overlay_keys(key, overlay);
        }
        if !self.app.overlay.tab_completions.is_empty() {
            return self.handle_tab_completion(key);
        }
        if self.app.overlay.slash_visible {
            return self.handle_slash_keys(key);
        }
        self.handle_normal_input(key)
    }

    // ── Phase 1: global shortcuts (Ctrl+C, Ctrl+Q, etc.) ──

    fn handle_global_shortcuts(&self, key: KeyEvent) -> bool {
        let m = key.modifiers;
        let has_ctrl = m.contains(KeyModifiers::CONTROL);
        let has_shift = m.contains(KeyModifiers::SHIFT);
        matches!(
            (key.code, has_ctrl, has_shift),
            (KeyCode::Char('c'), true, true)
        ) || matches!(
            (key.code, has_ctrl, has_shift),
            (KeyCode::Char('q'), true, false)
        ) || (matches!((key.code, has_ctrl), (KeyCode::Char('c'), true))
            && !has_shift
            && self.app.is_processing())
    }

    fn handle_global_action(&mut self, key: KeyEvent) -> Action {
        let m = key.modifiers;
        let has_ctrl = m.contains(KeyModifiers::CONTROL);
        let has_shift = m.contains(KeyModifiers::SHIFT);
        match (key.code, has_ctrl, has_shift) {
            (KeyCode::Char('c'), true, true) => {
                self.handle_copy()
            }
            (KeyCode::Char('c'), true, false) if self.app.is_processing() => {
                self.app.add_error("用户取消请求");
                Action::Continue
            }
            (KeyCode::Char('q'), true, false) => Action::Quit,
            _ => Action::Continue,
        }
    }

    // ── Phase 2: dismiss overlays with Esc ──

    fn handle_overlay_dismissals(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc if self.app.overlay.current == Some(Overlay::Feedback) => {
                self.app.overlay.close();
                return true;
            }
            KeyCode::Char('y') if self.app.overlay.current == Some(Overlay::Feedback) => {
                self.handle_submit_feedback(true);
                return true;
            }
            KeyCode::Char('n') if self.app.overlay.current == Some(Overlay::Feedback) => {
                self.handle_submit_feedback(false);
                return true;
            }
            KeyCode::Esc | KeyCode::Char('q') if self.app.overlay.selection_mode => {
                self.app.overlay.selection_mode = false;
                self.app.overlay.selected_message = None;
                return true;
            }
            KeyCode::Esc => match self.app.overlay.current {
                Some(Overlay::ToolList)
                | Some(Overlay::AgentList)
                | Some(Overlay::StatsHistory)
                | Some(Overlay::PluginList)
                | Some(Overlay::InfoPanel)
                | Some(Overlay::Config)
                | Some(Overlay::ThemePicker) => {
                    self.app.overlay.close();
                }
                Some(Overlay::Help) | Some(Overlay::Feedback) => {
                    self.app.overlay.close();
                }
                _ => {}
            },
            KeyCode::Enter if self.app.overlay.current == Some(Overlay::Help) => {
                self.app.overlay.close();
            }
            _ => {}
        }
        false
    }

    // ── Phase 3: selection mode ──

    fn handle_selection_mode(&mut self, key: KeyEvent) -> bool {
        if !self.app.overlay.selection_mode {
            return false;
        }

        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.overlay.selection_mode = false;
                self.app.overlay.selected_message = None;
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.handle_delete_selected_message();
                self.app.mark_dirty();
            }
            KeyCode::Up => {
                if let Some(idx) = self.app.overlay.selected_message
                    && idx > 0
                {
                    self.app.overlay.selected_message = Some(idx - 1);
                }
            }
            KeyCode::Down => {
                if let Some(idx) = self.app.overlay.selected_message
                    && idx + 1 < self.app.messages.len()
                {
                    self.app.overlay.selected_message = Some(idx + 1);
                }
            }
            KeyCode::Char(' ') => {
                if let Some(idx) = self.app.overlay.selected_message {
                    match self.app.messages.get(idx) {
                        Some(AppMessage::ToolCall { .. })
                            if !self.app.overlay.tool_call_expanded.remove(&idx) =>
                        {
                            self.app.overlay.tool_call_expanded.insert(idx);
                            self.app.mark_dirty();
                        }
                        Some(AppMessage::Assistant { reasoning, .. })
                            if !reasoning.is_empty()
                                && !self.app.overlay.reasoning_expanded.remove(&idx) =>
                        {
                            self.app.overlay.reasoning_expanded.insert(idx);
                            self.app.mark_dirty();
                        }
                        _ => {}
                    }
                }
            }
            _ => return false,
        }
        true
    }

    // ── Phase 4: overlay shortcut keys ──

    fn handle_overlay_shortcuts(&self, key: KeyEvent) -> bool {
        let is_processing = self.app.is_processing();
        let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);

        matches!(
            (key.code, has_ctrl, has_shift, is_processing),
            (KeyCode::Char('s'), true, false, _)
                | (KeyCode::Char('h'), true, false, _)
                | (KeyCode::Char('i'), true, false, _)
                | (KeyCode::Char('f'), true, false, false)
                | (KeyCode::Char('t'), true, false, _)
                | (KeyCode::Char('a'), true, false, _)
                | (KeyCode::Char('u'), true, true, _)
                | (KeyCode::Char('p'), true, true, _)
                | (KeyCode::Char('i'), true, true, _)
                | (KeyCode::Char('l'), true, false, _)
                | (KeyCode::Char('n'), true, false, _)
                | (KeyCode::Char('r'), true, false, _)
                | (KeyCode::Char('p'), true, false, _)
                | (KeyCode::Char('e'), true, false, _)
        )
    }

    fn handle_overlay_action(&mut self, key: KeyEvent) -> Action {
        let m = key.modifiers;
        let has_ctrl = m.contains(KeyModifiers::CONTROL);
        let has_shift = m.contains(KeyModifiers::SHIFT);
        match (key.code, has_ctrl, has_shift) {
            (KeyCode::Char('s'), true, false) if !self.app.messages.is_empty() => {
                if self.app.overlay.current == Some(Overlay::AgentList) {
                    let agent_ids: Vec<&String> = self.app.config.agents.keys().collect();
                    if !agent_ids.is_empty() {
                        let idx = self
                            .app
                            .overlay
                            .agent_picker_index
                            .min(agent_ids.len().saturating_sub(1));
                        if let Some(target_id) = agent_ids.get(idx).map(|s| s.as_str()) {
                            self.app.current_agent = target_id.to_string();
                        }
                    }
                } else {
                    self.app.overlay.selection_mode = !self.app.overlay.selection_mode;
                    self.app.overlay.selected_message = if self.app.overlay.selection_mode {
                        Some(self.app.messages.len().saturating_sub(1))
                    } else {
                        None
                    };
                }
            }
            (KeyCode::Char('h'), true, false) => {
                self.app.overlay.toggle(Overlay::Help);
            }
            (KeyCode::Char('i'), true, false) => {
                self.app.overlay.toggle(Overlay::Config);
            }
            (KeyCode::Char('f'), true, false) if !self.app.is_processing() => {
                self.app.overlay.toggle(Overlay::Feedback);
            }
            (KeyCode::Char('t'), true, false) => {
                self.app.overlay.toggle(Overlay::ToolList);
            }
            (KeyCode::Char('a'), true, false) => {
                self.app.overlay.toggle(Overlay::AgentList);
            }
            (KeyCode::Char('u'), true, true) => {
                if self.app.overlay.current == Some(Overlay::StatsHistory) {
                    self.app.overlay.close();
                } else {
                    self.app.overlay.show(Overlay::StatsHistory);
                    self.app.stats_history = self.app_core.stats_manager.daily_history(7);
                }
            }
            (KeyCode::Char('p'), true, true) => {
                if self.app.overlay.current == Some(Overlay::PluginList) {
                    self.app.overlay.close();
                } else {
                    self.app.overlay.show(Overlay::PluginList);
                    let store = self
                        .app_core
                        .agent_store
                        .skill_store_for(&self.app.current_agent);
                    self.app.skill_list = store.list_skills();
                    let plugin_mgr = crate::plugin::PluginManager::new();
                    self.app.plugin_list = plugin_mgr
                        .manifests
                        .iter()
                        .map(|m| app::PluginEntry {
                            name: m.plugin.name.clone(),
                            description: m.plugin.description.clone(),
                            enabled: plugin_mgr.is_enabled(&m.plugin.name),
                        })
                        .collect();
                }
            }
            (KeyCode::Char('i'), true, true) => {
                self.app.overlay.toggle(Overlay::InfoPanel);
            }
            (KeyCode::Char('l'), true, false) => {
                self.app.overlay.toggle(Overlay::SessionList);
                if self.app.overlay.is_overlay(Overlay::SessionList) {
                    self.app.overlay.session_list_index = 0;
                    self.app.overlay.session_list = self.app_core.session_mgr.sessions().to_vec();
                }
            }
            (KeyCode::Char('n'), true, false) => {
                return self.handle_new_session();
            }
            (KeyCode::Char('r'), true, false) => {
                self.app.overlay.toggle(Overlay::Sidebar);
            }
            (KeyCode::Char('p'), true, false) => {
                if self.app.overlay.current == Some(Overlay::AgentPicker) {
                    self.app.overlay.close();
                } else {
                    self.app.overlay.show(Overlay::AgentPicker);
                    self.app.overlay.agent_list = self.app_core.config.agent_ids();
                    self.app.overlay.agent_picker_index = self
                        .app
                        .overlay
                        .agent_list
                        .iter()
                        .position(|id| *id == self.app.current_agent)
                        .unwrap_or(0);
                }
            }
            (KeyCode::Char('e'), true, false) => {
                return self.handle_export_session();
            }
            _ => {}
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    // ── Phase 5: active overlay key handling ──

    fn handle_active_overlay_keys(&mut self, key: KeyEvent, overlay: Overlay) -> Action {
        match overlay {
            Overlay::SessionList => self.handle_session_list_keys(key),
            Overlay::Sidebar => self.handle_sidebar_keys(key),
            Overlay::AgentPicker => self.handle_agent_picker_keys(key),
            Overlay::ThemePicker => self.handle_theme_picker_keys(key),
            _ => Action::Continue,
        }
    }

    // ── Phase 6: tab completion ──

    fn handle_tab_completion(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.app.overlay.tab_completions.clear();
                self.app.overlay.tab_completion_index = 0;
            }
            KeyCode::Tab if !self.app.is_processing() && !self.app.input.text.is_empty() => {
                self.handle_tab_complete();
            }
            KeyCode::BackTab
                if !self.app.is_processing() && !self.app.overlay.tab_completions.is_empty() =>
            {
                self.handle_backtab_complete();
            }
            KeyCode::Tab if self.app.input.text.is_empty() && !self.app.is_processing() => {
                self.app.overlay.tab_completions.clear();
                self.app.overlay.toggle(Overlay::Config);
            }
            _ => {
                self.app.overlay.tab_completions.clear();
                self.app.overlay.tab_completion_index = 0;
                return self.handle_normal_input(key);
            }
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    // ── Phase 7: normal input ──

    fn handle_normal_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up if !self.app.overlay.selection_mode && !self.app.is_processing() => {
                if self.app.input.text.is_empty() {
                    self.app.scroll_up();
                } else if let Some(text) = self.app.input.navigate_up() {
                    if self.app.input.history_index.is_some()
                        && self.app.input.draft.is_empty()
                    {
                        self.app.input.draft = self.app.input.text.clone();
                    }
                    self.app.input.text = text;
                    self.app.input.move_cursor_end();
                }
                self.app.mark_overlay_dirty();
            }
            KeyCode::Down if !self.app.overlay.selection_mode && !self.app.is_processing() => {
                if self.app.input.text.is_empty() {
                    self.app.scroll_down();
                } else if let Some(text) = self.app.input.navigate_down() {
                    self.app.input.text = text;
                    self.app.input.move_cursor_end();
                } else {
                    if !self.app.input.draft.is_empty() {
                        self.app.input.text = std::mem::take(&mut self.app.input.draft);
                    } else {
                        self.app.input.text.clear();
                    }
                    self.app.input.cursor = self.app.input.text.len();
                }
                self.app.mark_overlay_dirty();
            }
            KeyCode::Enter => return self.handle_enter_key(key),
            _ => return self.handle_editing_key(key),
        }
        Action::Continue
    }

    fn handle_enter_key(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::ALT) {
            if !self.app.is_processing() {
                self.app.insert_char('\n');
            }
            return Action::Continue;
        }
        if !self.app.input.text.is_empty() && !self.app.is_processing() {
            let text = std::mem::take(&mut self.app.input.text);
            self.app.input.cursor = 0;
            self.app.commit_input_to_history(&text);
            self.app.add_user_message(&text);
            {
                let lm = self.app_core.agent_store.layered_memory_for_mut(&self.app.current_agent);
                lm.record_user_statement(&text);
            }
            self.app_core
                .session_mgr
                .append_message("user", &text, None);
            let msgs = self.app_core.build_messages_for(
                &self.app.messages,
                &text,
                &self.app.api_messages,
                self.app.reminder_text.as_deref(),
                &self.app.current_agent,
            );
            let recent: Vec<Value> = self
                .app
                .api_messages
                .as_deref()
                .map(|m| m.to_vec())
                .unwrap_or_default();
            self.app_core.spawn_chat_for(
                self.rt,
                self.llm_tx.clone(),
                msgs,
                &self.app.current_agent,
                &recent,
            );
        }
        Action::Continue
    }

    fn handle_editing_key(&mut self, key: KeyEvent) -> Action {
        if self.app.is_processing() {
            return Action::Continue;
        }
        match key.code {
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.input.delete_word_before_cursor();
                self.app.overlay.tab_completions.clear();
            }
            KeyCode::Backspace if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if !self.app.input.text.is_empty() {
                    self.app.delete_before_cursor();
                    self.app.overlay.tab_completions.clear();
                    if self.app.overlay.slash_visible
                        && !self.app.input.text.starts_with('/')
                    {
                        self.app.overlay.slash_visible = false;
                        self.app.overlay.slash_index = 0;
                    }
                }
            }
            KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.input.move_cursor_word_left();
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.input.move_cursor_word_right();
            }
            KeyCode::Left => {
                self.app.move_cursor_left();
            }
            KeyCode::Right => {
                self.app.move_cursor_right();
            }
            KeyCode::Home => {
                self.app.input.move_cursor_home();
            }
            KeyCode::End => {
                self.app.input.move_cursor_end();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.input.delete_to_line_start();
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.input.delete_to_line_end();
            }
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.input.undo();
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.input.redo();
            }
            KeyCode::Char(c) => {
                self.app.overlay.tab_completions.clear();
                self.app.insert_char(c);
                if self.app.input.text == "/" {
                    self.app.overlay.slash_visible = true;
                    self.app.overlay.slash_index = 0;
                }
            }
            _ => {}
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    // ── Copy ──

    fn handle_copy(&mut self) -> Action {
        let content = if self.app.overlay.selection_mode {
            self.app.overlay.selected_message.and_then(|idx| {
                self.app.messages.get(idx).map(|m| match m {
                    AppMessage::User { text } => text.clone(),
                    AppMessage::Assistant { text, .. } => text.clone(),
                    AppMessage::ToolCall {
                        name, args, result, ..
                    } => format!("Tool: {}\nArgs: {}\nResult: {}", name, args, result),
                    AppMessage::Error { text } => text.clone(),
                    AppMessage::Evaluation { tool, issues, .. } => {
                        format!("Tool Evaluation: {} | Issues: {}", tool, issues.join("; "))
                    }
                    _ => String::new(),
                })
            })
        } else {
            self.app.messages.iter().rev().find_map(|m| match m {
                AppMessage::Assistant { text, .. } if !text.is_empty() => Some(text.clone()),
                _ => None,
            })
        };
        if let Some(content) = content {
            if crate::tui::clipboard::copy_to_clipboard(&content) {
                self.app.overlay.copy_feedback = Some(("✓ 已复制".to_string(), std::time::Instant::now()));
            } else {
                self.app.overlay.copy_feedback = Some(("✗ 复制失败".to_string(), std::time::Instant::now()));
            }
        } else {
            self.app.overlay.copy_feedback = Some(("无内容可复制".to_string(), std::time::Instant::now()));
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    fn handle_delete_selected_message(&mut self) {
        if let Some(idx) = self.app.overlay.selected_message {
            let idx = idx.min(self.app.messages.len().saturating_sub(1));
            self.app.messages.remove(idx);
            self.app.message_timestamps.remove(idx);

            let tc = std::mem::take(&mut self.app.overlay.tool_call_expanded);
            self.app.overlay.tool_call_expanded = tc
                .into_iter()
                .filter(|&i| i != idx)
                .map(|i| if i > idx { i - 1 } else { i })
                .collect();

            let re = std::mem::take(&mut self.app.overlay.reasoning_expanded);
            self.app.overlay.reasoning_expanded = re
                .into_iter()
                .filter(|&i| i != idx)
                .map(|i| if i > idx { i - 1 } else { i })
                .collect();

            if idx >= self.app.messages.len() {
                self.app.overlay.selected_message = if self.app.messages.is_empty() {
                    None
                } else {
                    Some(self.app.messages.len() - 1)
                };
            }
            self.app.mark_dirty();
        }
    }

    fn handle_submit_feedback(&mut self, positive: bool) {
        self.app.overlay.close();
        self.app.messages.push(app::Message::Feedback {
            positive,
            message: None,
        });
        self.app
            .message_timestamps
            .push(chrono::Local::now().naive_local());
        self.app.mark_dirty();
        if let Some(sid) = self
            .app_core
            .session_mgr
            .current_id()
            .map(|s| s.to_string())
        {
            self.app_core
                .agent_store
                .memory_for_mut(&self.app.current_agent)
                .record_session_feedback(&sid, positive);
            self.app_core
                .agent_store
                .memory_for_mut(&self.app.current_agent)
                .flush();
        }
    }

    fn handle_new_session(&mut self) -> Action {
        if let Some(old_id) = self
            .app_core
            .session_mgr
            .current_id()
            .map(|id| id.to_string())
        {
            crate::tui::clipboard::save_session_messages(
                &self.app_core.session_mgr,
                &old_id,
                &self.app.messages,
                self.app.api_messages.as_deref(),
            );
        }
        self.app_core.session_mgr.create_session();
        self.app.reset_for_new_session();
        self.app.overlay.close();
        Action::Continue
    }

    // ── Session list ──

    fn handle_session_list_keys(&mut self, key: KeyEvent) -> Action {
        let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let filtered = self.app.overlay.filtered_sessions();
        let filtered_len = filtered.len();
        let filtered_max = filtered_len.saturating_sub(1);
        match key.code {
            KeyCode::Char('d') if has_ctrl => {
                self.app.overlay.session_confirm_delete = true;
            }
            KeyCode::Char('r') if has_ctrl => {
                if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
                    self.app.overlay.session_rename_buf = meta.title.clone();
                }
            }
            KeyCode::Char('y') if self.app.overlay.session_confirm_delete => {
                if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
                    let id = meta.id.clone();
                    let is_current = self
                        .app_core
                        .session_mgr
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
                    self.app.overlay.close();
                }
            }
            KeyCode::Up => {
                self.app.overlay.session_list_index =
                    self.app.overlay.session_list_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.app.overlay.session_list_index < filtered_max {
                    self.app.overlay.session_list_index += 1;
                }
            }
            KeyCode::PageUp => {
                let step = 10.min(self.app.overlay.session_list_index);
                self.app.overlay.session_list_index -= step;
            }
            KeyCode::PageDown => {
                self.app.overlay.session_list_index =
                    (self.app.overlay.session_list_index + 10).min(filtered_max);
            }
            KeyCode::Home => {
                self.app.overlay.session_list_index = 0;
            }
            KeyCode::End => {
                self.app.overlay.session_list_index = filtered_max;
            }
            KeyCode::Char('/') if !self.app.overlay.session_search_mode => {
                self.app.overlay.session_search_mode = true;
                self.app.overlay.session_search.clear();
            }
            KeyCode::Char(c) if !self.app.overlay.session_rename_buf.is_empty() => {
                self.app.overlay.session_rename_buf.push(c);
            }
            KeyCode::Char(c) if self.app.overlay.session_search_mode => {
                self.app.overlay.session_search.push(c);
                self.app.overlay.session_list_index = 0;
            }
            KeyCode::Backspace if !self.app.overlay.session_rename_buf.is_empty()
                && !self.app.overlay.session_search_mode =>
            {
                self.app.overlay.session_rename_buf.pop();
            }
            KeyCode::Backspace if self.app.overlay.session_search_mode => {
                self.app.overlay.session_search.pop();
                self.app.overlay.session_list_index = 0;
            }
            KeyCode::Enter => {
                return self.handle_session_enter(&filtered);
            }
            _ => {}
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    fn handle_session_enter(&mut self, filtered: &[crate::session::SessionMeta]) -> Action {
        if !self.app.overlay.session_rename_buf.is_empty() {
            if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
                let title = std::mem::take(&mut self.app.overlay.session_rename_buf);
                if !title.trim().is_empty() {
                    self.app_core
                        .session_mgr
                        .rename_session(&meta.id, title.trim());
                }
                self.app.overlay.session_list = self.app_core.session_mgr.sessions().to_vec();
            } else {
                self.app.overlay.session_rename_buf.clear();
            }
            self.app.mark_dirty();
            return Action::Continue;
        }

        if self.app.overlay.session_search_mode {
            self.app.overlay.session_search_mode = false;
        }

        if let Some(meta) = filtered.get(self.app.overlay.session_list_index) {
            let new_id = meta.id.clone();
            let is_current = self
                .app_core
                .session_mgr
                .current_id()
                .map(|id| id == new_id)
                .unwrap_or(false);
            if !is_current {
                let old_id = self
                    .app_core
                    .session_mgr
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
                let loaded = self.app_core.session_mgr.load_app_messages(&new_id, 200);
                self.app.messages = loaded;
                self.app.sync_message_timestamps();
                self.app.api_messages = self.app_core.session_mgr.load_api_messages(&new_id);
                self.app.tool_call_count = 0;
                self.app.status_text.clear();
                self.app.token_usage = None;
                self.app.plan_steps = self.app_core.session_mgr.load_plan_steps(&new_id);
                self.app.scroll_lines = 0;
                self.app.max_scroll = 0;
                self.app.overlay.tool_call_expanded.clear();
                self.app.overlay.reasoning_expanded.clear();
                self.app.mark_dirty();
            }
        }
        self.app.overlay.close();
        self.app.mark_dirty();
        Action::Continue
    }

    // ── Sidebar ──

    fn handle_sidebar_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc if self.app.overlay.sidebar_body_idx.is_some() => {
                self.app.overlay.sidebar_body_idx = None;
            }
            KeyCode::Esc => {
                self.app.overlay.close();
            }
            KeyCode::Up if self.app.overlay.sidebar_body_idx.is_none() => {
                self.app.overlay.sidebar_selected =
                    self.app.overlay.sidebar_selected.saturating_sub(1);
            }
            KeyCode::Down if self.app.overlay.sidebar_body_idx.is_none() => {
                let max = self.app.http_logs.len().saturating_sub(1);
                if self.app.overlay.sidebar_selected < max {
                    self.app.overlay.sidebar_selected += 1;
                }
            }
            KeyCode::Enter
                if self.app.overlay.sidebar_body_idx.is_none()
                    && !self.app.http_logs.is_empty() =>
            {
                self.app.overlay.sidebar_body_idx = Some(self.app.overlay.sidebar_selected);
                self.app.overlay.sidebar_body_scroll = 0;
            }
            KeyCode::Up if self.app.overlay.sidebar_body_idx.is_some() => {
                self.app.overlay.sidebar_body_scroll =
                    self.app.overlay.sidebar_body_scroll.saturating_sub(3);
            }
            KeyCode::Down if self.app.overlay.sidebar_body_idx.is_some() => {
                if let Some(idx) = self.app.overlay.sidebar_body_idx
                    && let Some(log) = self.app.http_logs.get(idx)
                {
                    let content_lines = log.request_body.lines().count() * 3;
                    let max = content_lines.saturating_sub(1);
                    self.app.overlay.sidebar_body_scroll =
                        (self.app.overlay.sidebar_body_scroll + 3).min(max);
                }
            }
            _ => {}
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    // ── Agent picker ──

    fn handle_agent_picker_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up => {
                self.app.overlay.agent_picker_index =
                    self.app.overlay.agent_picker_index.saturating_sub(1);
                self.app.mark_overlay_dirty();
            }
            KeyCode::Down => {
                let max = self.app.overlay.agent_list.len().saturating_sub(1);
                if self.app.overlay.agent_picker_index < max {
                    self.app.overlay.agent_picker_index += 1;
                }
                self.app.mark_overlay_dirty();
            }
            KeyCode::Enter => {
                let agent_id = self
                    .app
                    .overlay
                    .agent_list
                    .get(self.app.overlay.agent_picker_index)
                    .cloned();
                if let Some(ref agent_id) = agent_id
                    && *agent_id != self.app.current_agent
                {
                    if let Some(old_id) = self
                        .app_core
                        .session_mgr
                        .current_id()
                        .map(|id| id.to_string())
                    {
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
                    self.app_core
                        .agent_store
                        .memory_for_mut(agent_id)
                        .analyze_sessions(
                            self.app_core.session_mgr.sessions(),
                            &self.app_core.session_mgr,
                        );
                }
                self.app.overlay.close();
                self.app.mark_overlay_dirty();
            }
            KeyCode::Esc => {
                self.app.overlay.close();
                self.app.mark_overlay_dirty();
            }
            _ => {}
        }
        Action::Continue
    }

    fn handle_export_session(&mut self) -> Action {
        if self.app.messages.is_empty() {
            self.app.overlay.copy_feedback = Some(("无消息可导出".to_string(), std::time::Instant::now()));
            return Action::Continue;
        }

        let mut md = String::new();
        md.push_str(&format!(
            "# 会话导出 - {}\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M")
        ));
        md.push_str(&format!("Agent: {}\n", self.app.current_agent));
        md.push_str(&format!("模型: {}\n\n", self.app.config.model));
        md.push_str("---\n\n");

        for msg in &self.app.messages {
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
                self.app.overlay.copy_feedback = Some((format!("✓ 已导出: {}", filename), std::time::Instant::now()));
            }
            Err(e) => {
                self.app.overlay.copy_feedback = Some((format!("✗ 导出失败: {}", e), std::time::Instant::now()));
            }
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    // ── Slash command panel ──

    fn handle_slash_keys(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.app.overlay.slash_visible = false;
                self.app.overlay.slash_index = 0;
                self.app.input.text.clear();
                self.app.input.cursor = 0;
                return Action::Continue;
            }
            KeyCode::Up => {
                self.app.overlay.slash_index =
                    self.app.overlay.slash_index.saturating_sub(1);
                return Action::Continue;
            }
            KeyCode::Down => {
                let max = self.slash_match_count().saturating_sub(1);
                if self.app.overlay.slash_index < max {
                    self.app.overlay.slash_index += 1;
                }
                return Action::Continue;
            }
            KeyCode::Enter => {
                return self.handle_slash_execute();
            }
            KeyCode::Tab => {
                return self.handle_slash_execute();
            }
            KeyCode::Backspace => {
                self.app.input.delete_before_cursor();
                if !self.app.input.text.starts_with('/') {
                    self.app.overlay.slash_visible = false;
                    self.app.overlay.slash_index = 0;
                } else {
                    self.app.overlay.slash_index = 0;
                }
                return Action::Continue;
            }
            KeyCode::Char(c) => {
                self.app.insert_char(c);
                self.app.overlay.slash_index = 0;
                return Action::Continue;
            }
            _ => return Action::Continue,
        }
    }

    fn slash_match_count(&self) -> usize {
        let query = if self.app.input.text.starts_with('/') {
            &self.app.input.text
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
                cmd.name.starts_with(&q)
                    || (query.len() > 1 && cmd.desc.contains(&query[1..]))
            })
            .count()
    }

    fn handle_slash_execute(&mut self) -> Action {
        let query = if self.app.input.text.starts_with('/') {
            &self.app.input.text
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
                cmd.name.starts_with(&q)
                    || (query.len() > 1 && cmd.desc.contains(&query[1..]))
            })
            .collect();

        let idx = self.app.overlay.slash_index.min(matches.len().saturating_sub(1));
        let action = matches.get(idx).map(|cmd| cmd.action);

        self.app.input.text.clear();
        self.app.input.cursor = 0;
        self.app.overlay.slash_visible = false;
        self.app.overlay.slash_index = 0;

        match action {
            Some(crate::app::SlashAction::Help) => {
                self.app.overlay.show(Overlay::Help);
            }
            Some(crate::app::SlashAction::Sessions) => {
                self.app.overlay.show(Overlay::SessionList);
                self.app.overlay.session_list_index = 0;
                self.app.overlay.session_list =
                    self.app_core.session_mgr.sessions().to_vec();
            }
            Some(crate::app::SlashAction::New) => {
                return self.handle_new_session();
            }
            Some(crate::app::SlashAction::Agent) => {
                self.app.overlay.show(Overlay::AgentPicker);
                self.app.overlay.agent_list = self.app_core.config.agent_ids();
                self.app.overlay.agent_picker_index = self
                    .app
                    .overlay
                    .agent_list
                    .iter()
                    .position(|id| *id == self.app.current_agent)
                    .unwrap_or(0);
            }
            Some(crate::app::SlashAction::Agents) => {
                self.app.overlay.show(Overlay::AgentList);
            }
            Some(crate::app::SlashAction::Tools) => {
                self.app.overlay.show(Overlay::ToolList);
            }
            Some(crate::app::SlashAction::Sidebar) => {
                self.app.overlay.toggle(Overlay::Sidebar);
            }
            Some(crate::app::SlashAction::Stats) => {
                self.app.overlay.show(Overlay::StatsHistory);
                self.app.stats_history = self.app_core.stats_manager.daily_history(7);
            }
            Some(crate::app::SlashAction::Plugins) => {
                self.app.overlay.show(Overlay::PluginList);
                let store = self
                    .app_core
                    .agent_store
                    .skill_store_for(&self.app.current_agent);
                self.app.skill_list = store.list_skills();
                let plugin_mgr = crate::plugin::PluginManager::new();
                self.app.plugin_list = plugin_mgr
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
                self.app.overlay.show(Overlay::Config);
            }
            Some(crate::app::SlashAction::Export) => {
                return self.handle_export_session();
            }
            Some(crate::app::SlashAction::Feedback) => {
                self.app.overlay.show(Overlay::Feedback);
            }
            Some(crate::app::SlashAction::Info) => {
                self.app.overlay.show(Overlay::InfoPanel);
            }
            Some(crate::app::SlashAction::Select) => {
                if !self.app.messages.is_empty() {
                    self.app.overlay.selection_mode = true;
                    self.app.overlay.selected_message =
                        Some(self.app.messages.len().saturating_sub(1));
                }
            }
            Some(crate::app::SlashAction::Clear) => {
                self.app.messages.clear();
                self.app.message_timestamps.clear();
                self.app.api_messages = None;
                self.app.tool_call_count = 0;
                self.app.status_text.clear();
                self.app.scroll_lines = 0;
                self.app.max_scroll = 0;
                self.app.overlay.tool_call_expanded.clear();
                self.app.overlay.reasoning_expanded.clear();
                self.app.mark_dirty();
            }
            Some(crate::app::SlashAction::Compact) => {
                if let Some(_sid) = self.app_core.session_mgr.current_id().map(|s| s.to_string())
                {
                    let memory = self
                        .app_core
                        .agent_store
                        .memory_for_mut(&self.app.current_agent);
                    memory.analyze_sessions(
                        self.app_core.session_mgr.sessions(),
                        &self.app_core.session_mgr,
                    );
                    memory.flush();
                    self.app.overlay.copy_feedback =
                        Some(("✓ 上下文已压缩".to_string(), std::time::Instant::now()));
                }
            }
            Some(crate::app::SlashAction::Theme) => {
                self.app.overlay.show(Overlay::ThemePicker);
                self.app.overlay.theme_index = crate::theme::BUILT_IN_THEMES
                    .iter()
                    .position(|t| {
                        let theme = &self.app.config.theme;
                        theme.primary.as_deref() == Some(t.primary)
                            && theme.background.as_deref() == Some(t.background)
                    })
                    .unwrap_or(0);
            }
            None => {}
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    // ── Theme picker ──

    fn handle_theme_picker_keys(&mut self, key: KeyEvent) -> Action {
        let max = crate::theme::BUILT_IN_THEMES.len().saturating_sub(1);
        match key.code {
            KeyCode::Up => {
                self.app.overlay.theme_index =
                    self.app.overlay.theme_index.saturating_sub(1);
                self.apply_theme_preview();
            }
            KeyCode::Down => {
                if self.app.overlay.theme_index < max {
                    self.app.overlay.theme_index += 1;
                }
                self.apply_theme_preview();
            }
            KeyCode::Enter => {
                self.apply_theme_preview();
                self.save_theme();
                self.app.overlay.close();
            }
            KeyCode::Esc => {
                self.app.overlay.close();
            }
            _ => {}
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    fn apply_theme_preview(&mut self) {
        if let Some(preset) =
            crate::theme::BUILT_IN_THEMES.get(self.app.overlay.theme_index)
        {
            self.app.config.theme =
                crate::theme::Theme::from_preset(preset.name)
                    .unwrap_or_default();
        }
    }

    fn save_theme(&self) {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return,
        };
        let theme_path = home.join(".i-rs").join("claw").join("theme.json");
        if let Err(e) = self.app.config.theme.save(&theme_path) {
            tracing::warn!("保存主题失败: {}", e);
        }
    }

    // ── Export ──



    fn handle_tab_complete(&mut self) {
        let completions = crate::completion::get_completions(
            &self.app_core.config,
            &self.app.input.text,
            self.app.input.cursor,
        );
        if !completions.is_empty() {
            if self.app.overlay.tab_completions.is_empty() {
                self.app.overlay.tab_completions = completions;
                self.app.overlay.tab_completion_index = 0;
            } else {
                self.app.overlay.tab_completion_index = (self.app.overlay.tab_completion_index + 1)
                    % self.app.overlay.tab_completions.len();
            }
            let selected = &self.app.overlay.tab_completions[self.app.overlay.tab_completion_index];
            let before = &self.app.input.text[..self.app.input.cursor];
            let after = &self.app.input.text[self.app.input.cursor..];
            let word_start = before
                .rfind(|c: char| c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(0);
            self.app.input.text = format!("{}{} {}", &before[..word_start], selected, after);
            self.app.input.cursor = word_start + selected.len() + 1;
        } else {
            self.app.overlay.tab_completions.clear();
            self.app.overlay.tab_completion_index = 0;
        }
        self.app.mark_overlay_dirty();
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
        let word_start = before
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        self.app.input.text = format!("{}{} {}", &before[..word_start], selected, after);
        self.app.input.cursor = word_start + selected.len() + 1;
        self.app.mark_overlay_dirty();
    }
}

pub struct MouseEventHandler<'a> {
    pub app: &'a mut App,
}

impl<'a> MouseEventHandler<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    pub fn handle(&mut self, mouse: MouseEvent) {
        if self.app.overlay.sidebar_body_idx.is_some() {
            match mouse.kind {
                MouseEventKind::ScrollDown => {
                    if let Some(idx) = self.app.overlay.sidebar_body_idx
                        && let Some(log) = self.app.http_logs.get(idx)
                    {
                        let max = log.request_body.lines().count() * 3;
                        self.app.overlay.sidebar_body_scroll =
                            (self.app.overlay.sidebar_body_scroll + 3).min(max);
                    }
                }
                MouseEventKind::ScrollUp => {
                    self.app.overlay.sidebar_body_scroll =
                        self.app.overlay.sidebar_body_scroll.saturating_sub(3);
                }
                _ => {}
            }
        } else if !self.app.is_processing()
            && self.app.overlay.current.is_none()
        {
            match mouse.kind {
                MouseEventKind::ScrollDown => self.app.scroll_down(),
                MouseEventKind::ScrollUp => self.app.scroll_up(),
                _ => {}
            }
        }
    }
}

type AppMessage = app::Message;
