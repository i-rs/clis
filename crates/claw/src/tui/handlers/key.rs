use crate::app::{self, App, Overlay};
use crate::ui::chat_api::ComponentOp;
use crate::core;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_interact::events::{is_space};
use serde_json::Value;
use tokio::sync::mpsc;

use super::{Action, AppMessage};

pub struct KeyEventHandler<'a> {
    pub app: &'a mut App,
    pub app_core: &'a mut core::AppCore,
    pub rt: &'a tokio::runtime::Runtime,
    pub llm_tx: &'a mpsc::UnboundedSender<crate::llm::LlmEvent>,
}

impl<'a> KeyEventHandler<'a> {
    pub fn new(
        app: &'a mut App,
        app_core: &'a mut core::AppCore,
        rt: &'a tokio::runtime::Runtime,
        llm_tx: &'a mpsc::UnboundedSender<crate::llm::LlmEvent>,
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
            return super::overlay::handle_active_overlay_keys(self, key, overlay);
        }
        if !self.app.overlay.tab_completions.is_empty() {
            return self.handle_tab_completion(key);
        }
        if self.app.overlay.slash_visible {
            return super::overlay::handle_slash_keys(self, key);
        }
        self.handle_normal_input(key)
    }

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

    fn handle_overlay_dismissals(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc if self.app.overlay.current == Some(Overlay::Feedback) => {
                self.app.overlay.close();
                self.app.mark_overlay_dirty();
                return true;
            }
            KeyCode::Char('y') if self.app.overlay.current == Some(Overlay::Feedback) => {
                self.handle_submit_feedback(true);
                self.app.mark_overlay_dirty();
                return true;
            }
            KeyCode::Char('n') if self.app.overlay.current == Some(Overlay::Feedback) => {
                self.handle_submit_feedback(false);
                self.app.mark_overlay_dirty();
                return true;
            }
            KeyCode::Esc | KeyCode::Char('q') if self.app.overlay.selection_mode => {
                self.app.overlay.selection_mode = false;
                self.app.overlay.selected_message = None;
                self.app.mark_overlay_dirty();
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
        self.app.mark_overlay_dirty();
        false
    }

    fn handle_selection_mode(&mut self, key: KeyEvent) -> bool {
        if !self.app.overlay.selection_mode {
            return false;
        }

        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.overlay.selection_mode = false;
                self.app.overlay.selected_message = None;
                self.app.mark_overlay_dirty();
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
                    self.app.scroll_to_selected();
                }
                self.app.mark_overlay_dirty();
            }
            KeyCode::Down => {
                if let Some(idx) = self.app.overlay.selected_message
                    && idx + 1 < self.app.messages.len()
                {
                    self.app.overlay.selected_message = Some(idx + 1);
                    self.app.scroll_to_selected();
                }
                self.app.mark_overlay_dirty();
            }
            _ if is_space(&key) => {
                if let Some(idx) = self.app.overlay.selected_message {
                    // Only trigger a state change when the selected
                    // message has a component that opts into toggling
                    // (i.e. a tool call card or an assistant with
                    // reasoning). This is the keyboard counterpart of
                    // the hit-test used by `mouse::handle_click`.
                    let can_toggle = self
                        .app
                        .components
                        .get(idx)
                        .map(|c| c.borrow().clickable())
                        .unwrap_or(false);
                    if can_toggle {
                        self.app.toggle_component_at(idx);
                        self.app.rebuild_heights_approx();
                        self.app.scroll_to_selected();
                    }
                }
            }
            // Sub-toggles for tool-call cards: collapse Args / Result
            // independently without collapsing the whole card. Only
            // takes effect on a ToolCallCard (other components ignore
            // these ops via the default `apply` no-op).
            KeyCode::Char('a') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(idx) = self.app.overlay.selected_message {
                    self.app.apply_to_component(idx, ComponentOp::ToggleArgs);
                    self.app.rebuild_heights_approx();
                    self.app.scroll_to_selected();
                    self.app.mark_dirty();
                }
            }
            KeyCode::Char('r') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(idx) = self.app.overlay.selected_message {
                    self.app.apply_to_component(idx, ComponentOp::ToggleResult);
                    self.app.rebuild_heights_approx();
                    self.app.scroll_to_selected();
                    self.app.mark_dirty();
                }
            }
            _ => return false,
        }
        true
    }

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
                    self.app.today_stats = self.app_core.stats_manager.today_summary();
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
                    self.app.overlay.invalidate_session_cache();
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
                return super::overlay::handle_export_session(self);
            }
            _ => {}
        }
        self.app.mark_overlay_dirty();
        Action::Continue
    }

    fn handle_normal_input(&mut self, key: KeyEvent) -> Action {
        let can_scroll = !self.app.overlay.selection_mode && !self.app.is_processing();
        match key.code {
            KeyCode::Up if can_scroll => {
                if self.app.input.text.is_empty() {
                    self.app.scroll_up_one();
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
            KeyCode::Down if can_scroll => {
                if self.app.input.text.is_empty() {
                    self.app.scroll_down_one();
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
            KeyCode::PageUp if can_scroll => {
                self.app.scroll_page(1);
                self.app.mark_overlay_dirty();
            }
            KeyCode::PageDown if can_scroll => {
                self.app.scroll_page(-1);
                self.app.mark_overlay_dirty();
            }
            KeyCode::Home if can_scroll && self.app.input.text.is_empty() => {
                self.app.scroll_lines = self.app.max_scroll;
                self.app.stick_to_bottom = false;
                self.app.mark_overlay_dirty();
            }
            KeyCode::End if can_scroll && self.app.input.text.is_empty() => {
                self.app.scroll_lines = 0;
                self.app.stick_to_bottom = true;
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
            // Drop the matching component too — state lives there
            // now, so we just hand the slot to the caller.
            self.app.components.remove(idx);

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
        if let Some(sid) = self.app_core.session_mgr.current_id() {
            self.app_core
                .agent_store
                .memory_for_mut(&self.app.current_agent)
                .record_session_feedback(sid, positive);
            self.app_core
                .agent_store
                .memory_for_mut(&self.app.current_agent)
                .flush();
        }
    }

    pub(crate) fn handle_new_session(&mut self) -> Action {
        if let Some(old_id) = self.app_core.session_mgr.current_id() {
            crate::tui::clipboard::save_session_messages(
                &self.app_core.session_mgr,
                old_id,
                &self.app.messages,
                self.app.api_messages.as_deref(),
            );
        }
        self.app_core.session_mgr.create_session();
        self.app.reset_for_new_session();
        self.app.overlay.close();
        Action::Continue
    }

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
