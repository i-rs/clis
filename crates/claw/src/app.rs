use crate::config::Config;
use crate::stats::TodaySummary;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Clone)]
pub struct PluginEntry {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

/// A step in the LLM's execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub description: String,
    pub done: bool,
}

/// Record of an HTTP request to the LLM API.
#[derive(Debug, Clone)]
pub struct HttpLog {
    pub timestamp: String,       // formatted local time
    pub status: u16,             // HTTP status code
    pub duration_ms: u64,        // total request + streaming time
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub error: Option<String>,   // non-empty on failure
    pub request_body: String,    // assembled JSON body sent to LLM
}

#[derive(Clone)]
pub enum Message {
    User { text: String },
    Assistant { text: String, reasoning: String },
    ToolCall {
        name: String,
        args: String,
        result: String,
        step: usize,
        total_steps: usize,
    },
    Error { text: String },
}

/// Input editing state (input text, cursor, history).
#[derive(Clone)]
pub struct InputState {
    pub text: String,
    pub cursor: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.text.clone());
        if self.undo_stack.len() > 100 {
            self.undo_stack.drain(..50);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.text.clone());
            self.text = prev;
            self.cursor = self.cursor.min(self.text.len());
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.text.clone());
            self.text = next;
            self.cursor = self.cursor.min(self.text.len());
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.push_undo();
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.push_undo();
        let prev = self.text[..self.cursor].char_indices().next_back();
        if let Some((idx, _)) = prev {
            self.text.drain(idx..self.cursor);
            self.cursor = idx;
        }
    }

    pub fn delete_word_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.push_undo();
        let before = &self.text[..self.cursor];
        let trimmed = before.trim_end_matches(|c: char| c.is_whitespace());
        let word_start = trimmed
            .char_indices()
            .rev()
            .position(|(_, c)| !c.is_alphanumeric() && c != '_')
            .map(|p| {
                let idx = trimmed.len() - p - 1;
                let (_, c) = trimmed.char_indices().nth(idx).unwrap();
                idx + c.len_utf8()
            })
            .unwrap_or(0);
        self.text.drain(word_start..self.cursor);
        self.cursor = word_start;
    }

    pub fn delete_to_line_start(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.push_undo();
        self.text.drain(..self.cursor);
        self.cursor = 0;
    }

    pub fn delete_to_line_end(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        self.push_undo();
        self.text.drain(self.cursor..);
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = self.text[..self.cursor].char_indices().next_back();
        if let Some((idx, _)) = prev {
            self.cursor = idx;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let next = self.text[self.cursor..].char_indices().nth(1);
        if let Some((offset, _)) = next {
            self.cursor += offset;
        } else {
            self.cursor = self.text.len();
        }
    }

    pub fn move_cursor_word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let before = &self.text[..self.cursor];
        let trimmed = before.trim_end_matches(|c: char| c.is_whitespace());
        if trimmed.is_empty() {
            self.cursor = 0;
            return;
        }
        let new_pos = trimmed
            .char_indices()
            .rev()
            .position(|(_, c)| !c.is_alphanumeric() && c != '_')
            .map(|p| {
                let idx = trimmed.len() - p - 1;
                let (_, c) = trimmed.char_indices().nth(idx).unwrap();
                idx + c.len_utf8()
            })
            .unwrap_or(0);
        self.cursor = new_pos;
    }

    pub fn move_cursor_word_right(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let after = &self.text[self.cursor..];
        let trimmed = after.trim_start_matches(|c: char| c.is_whitespace());
        if trimmed.is_empty() {
            self.cursor = self.text.len();
            return;
        }
        let skip_ws = after.len() - trimmed.len();
        let word_end = trimmed
            .char_indices()
            .position(|(_, c)| !c.is_alphanumeric() && c != '_')
            .unwrap_or(trimmed.len());
        self.cursor = self.cursor + skip_ws + word_end;
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.text.len();
    }

    pub fn commit_to_history(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.history.last().map(|s| s.as_str()) != Some(text) {
            self.history.push(text.to_string());
            if self.history.len() > 50 {
                self.history.remove(0);
            }
        }
        self.history_index = None;
    }

    pub fn navigate_up(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }
        let new_idx = match self.history_index {
            Some(i) if i > 0 => i - 1,
            None => self.history.len().saturating_sub(1),
            _ => return None,
        };
        self.history_index = Some(new_idx);
        Some(self.history[new_idx].clone())
    }

    pub fn navigate_down(&mut self) -> Option<String> {
        match self.history_index {
            Some(i) if i + 1 < self.history.len() => {
                self.history_index = Some(i + 1);
                Some(self.history[i + 1].clone())
            }
            Some(_) => {
                self.history_index = None;
                None
            }
            None => None,
        }
    }
}

/// UI overlay state (menus, selections, feedback).
pub struct OverlayState {
    pub show_session_list: bool,
    pub session_list_index: usize,
    pub session_list: Vec<crate::session::SessionMeta>,
    pub session_search: String,
    pub session_search_mode: bool,
    pub session_rename_buf: String,
    pub session_confirm_delete: bool,
    pub show_sidebar: bool,
    pub sidebar_selected: usize,
    pub sidebar_body_idx: Option<usize>,
    pub sidebar_body_scroll: usize,
    pub show_agent_picker: bool,
    pub agent_picker_index: usize,
    pub agent_list: Vec<String>,
    pub selection_mode: bool,
    pub selected_message: Option<usize>,
    pub tool_call_expanded: HashSet<usize>,
    pub reasoning_expanded: HashSet<usize>,
    pub show_help: bool,
    pub show_config: bool,
    pub show_tool_list: bool,
    pub show_agent_list: bool,
    pub show_stats_history: bool,
    pub show_plugin_list: bool,
    pub copy_feedback: Option<String>,
    pub tab_completions: Vec<String>,
    pub tab_completion_index: usize,
}

impl OverlayState {
    /// Return the session list filtered by the current search query.
    /// Clones session metadata so the caller can freely mutate overlay state.
    pub fn filtered_sessions(&self) -> Vec<crate::session::SessionMeta> {
        let q = self.session_search.to_lowercase();
        if q.is_empty() {
            self.session_list.clone()
        } else {
            self.session_list
                .iter()
                .filter(|s| s.title.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    }

    pub fn new(agent_list: Vec<String>) -> Self {
        Self {
            show_session_list: false,
            session_list_index: 0,
            session_list: Vec::new(),
            session_search: String::new(),
            session_search_mode: false,
            session_rename_buf: String::new(),
            session_confirm_delete: false,
            show_sidebar: false,
            sidebar_selected: 0,
            sidebar_body_idx: None,
            sidebar_body_scroll: 0,
            show_agent_picker: false,
            agent_picker_index: 0,
            agent_list,
            selection_mode: false,
            selected_message: None,
            tool_call_expanded: HashSet::new(),
            reasoning_expanded: HashSet::new(),
            show_help: false,
            show_config: false,
            show_tool_list: false,
            show_agent_list: false,
            show_stats_history: false,
            show_plugin_list: false,
            copy_feedback: None,
            tab_completions: Vec::new(),
            tab_completion_index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Processing,
}

pub struct App {
    pub messages: Vec<Message>,
    pub message_timestamps: Vec<NaiveDateTime>,
    pub input: InputState,
    pub overlay: OverlayState,
    pub state: AppState,
    pub config: Config,
    pub tool_call_count: usize,
    /// Real-time status text shown in status bar (e.g. "思考中…", "正在调用工具…")
    pub status_text: String,
    /// Full API message list preserved across turns (includes tool call context)
    pub api_messages: Option<Vec<Value>>,
    /// Token usage from the last LLM response
    pub token_usage: Option<crate::llm::TokenUsage>,
    /// How many lines the user has scrolled up from the bottom (0 = bottom)
    pub scroll_lines: usize,
    /// HTTP request logs (newest first)
    pub http_logs: Vec<HttpLog>,
    /// Current reasoning text from LLM (DeepSeek chain-of-thought)
    pub current_reasoning: String,
    /// Proactive reminder text from i-rs remind (shown to LLM on next user message)
    pub reminder_text: Option<String>,
    /// Current execution plan steps (for plan-and-execute)
    pub plan_steps: Vec<PlanStep>,
    /// Current agent profile ID
    pub current_agent: String,
    pub today_stats: TodaySummary,
    pub stats_history: Vec<crate::stats::DailyStats>,
    pub skill_list: Vec<crate::skill_store::SkillEntry>,
    pub plugin_list: Vec<PluginEntry>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let agent_list = config.agent_ids();

        Self {
            messages: Vec::new(),
            message_timestamps: Vec::new(),
            input: InputState::new(),
            overlay: OverlayState::new(agent_list),
            state: AppState::Idle,
            config,
            tool_call_count: 0,
            status_text: String::new(),
            api_messages: None,
            token_usage: None,
            scroll_lines: 0,
            http_logs: Vec::new(),
            current_reasoning: String::new(),
            reminder_text: None,
            plan_steps: Vec::new(),
            current_agent: "default".to_string(),
            today_stats: TodaySummary::default(),
            stats_history: Vec::new(),
            skill_list: Vec::new(),
            plugin_list: Vec::new(),
        }
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.state, AppState::Processing)
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.overlay.copy_feedback.take();
        self.messages
            .push(Message::User { text: text.to_string() });
        self.message_timestamps.push(chrono::Local::now().naive_local());
        self.state = AppState::Processing;
        self.scroll_lines = 0;
        self.plan_steps.clear(); // Clear plan from previous turn
    }

    // ── Input cursor manipulation ──

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.overlay.copy_feedback.take();
        self.input.insert_char(c);
    }

    /// Delete the character before the cursor (Backspace).
    pub fn delete_before_cursor(&mut self) {
        self.input.delete_before_cursor();
    }

    pub fn move_cursor_left(&mut self) {
        self.input.move_cursor_left();
    }

    pub fn move_cursor_right(&mut self) {
        self.input.move_cursor_right();
    }

    #[allow(dead_code)]
    pub fn move_cursor_home(&mut self) {
        self.input.move_cursor_home();
    }

    #[allow(dead_code)]
    pub fn move_cursor_end(&mut self) {
        self.input.move_cursor_end();
    }

    /// Push text into input history (max 50 entries), reset history index.
    pub fn commit_input_to_history(&mut self, text: &str) {
        self.input.commit_to_history(text);
        self.scroll_lines = 0;
    }

    /// Navigate up in input history: restore previous input.
    /// Returns the text to put in `input`, or None if already at start.
    #[allow(dead_code)]
    pub fn navigate_history_up(&mut self) -> Option<String> {
        self.input.navigate_up()
    }

    /// Navigate down in input history: go to next input, or clear if at end.
    /// Returns Some(text) to put in `input`, or None to clear.
    #[allow(dead_code)]
    pub fn navigate_history_down(&mut self) -> Option<String> {
        self.input.navigate_down()
    }

    // =============================================
    // Message scroll
    // =============================================

    /// Scroll messages up (toward older messages) by ~3 lines.
    pub fn scroll_up(&mut self) {
        self.scroll_lines += 3;
    }

    /// Scroll messages down (toward newer messages) by ~3 lines.
    pub fn scroll_down(&mut self) {
        self.scroll_lines = self.scroll_lines.saturating_sub(3);
    }

    /// Update the real-time status text (shown in status bar)
    pub fn set_status(&mut self, text: &str) {
        self.status_text = text.to_string();
    }

    /// Start a new assistant message. If the last message is an empty assistant,
    /// reuse it instead of creating a new one.
    pub fn start_assistant_message(&mut self) {
        // Capture any accumulated reasoning into the last assistant message
        if !self.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.messages.last_mut() {
                reasoning.push_str(&self.current_reasoning);
        }
        self.current_reasoning.clear();
        let is_empty_assistant = matches!(
            self.messages.last(),
            Some(Message::Assistant { text, .. }) if text.is_empty()
        );
        if !is_empty_assistant {
            self.messages
                .push(Message::Assistant { text: String::new(), reasoning: String::new() });
            self.message_timestamps.push(chrono::Local::now().naive_local());
        }
    }

    pub fn append_assistant_text(&mut self, text: &str) {
        let last_is_assistant =
            matches!(self.messages.last_mut(), Some(Message::Assistant { .. }));
        if !last_is_assistant {
            self.start_assistant_message();
        }
        if let Some(Message::Assistant { text: t, .. }) = self.messages.last_mut() {
            t.push_str(text);
        }
    }

    pub fn add_tool_call(&mut self, name: &str, args: &str, result: &str, step: usize, total_steps: usize) {
        self.messages.push(Message::ToolCall {
            name: name.to_string(),
            args: args.to_string(),
            result: result.to_string(),
            step,
            total_steps,
        });
        self.message_timestamps.push(chrono::Local::now().naive_local());
        self.tool_call_count += 1;
    }

    pub fn add_http_log(&mut self, log: HttpLog) {
        self.http_logs.insert(0, log);
        if self.http_logs.len() > 50 {
            self.http_logs.pop();
        }
    }

    pub fn add_error(&mut self, text: &str) {
        // Capture any accumulated reasoning into the last assistant message
        if !self.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.messages.last_mut() {
                reasoning.push_str(&self.current_reasoning);
        }
        self.current_reasoning.clear();
        // Remove trailing empty assistant message (from NewRound before error)
        if let Some(Message::Assistant { text, .. }) = self.messages.last()
            && text.is_empty() {
                self.messages.pop();
            }
        self.messages
            .push(Message::Error { text: text.to_string() });
        // Reset API messages so the next request rebuilds from scratch
        self.api_messages = None;
        self.state = AppState::Idle;
        self.status_text.clear();
    }

    /// Finish processing and save the accumulated API messages for context preservation
    pub fn finish_processing(&mut self, api_messages: Option<Vec<Value>>) {
        // Capture any accumulated reasoning into the last assistant message
        if !self.current_reasoning.is_empty()
            && let Some(Message::Assistant { reasoning, .. }) = self.messages.last_mut() {
                reasoning.push_str(&self.current_reasoning);
        }
        self.current_reasoning.clear();
        // Remove trailing empty assistant message
        if let Some(Message::Assistant { text, .. }) = self.messages.last()
            && text.is_empty() {
                self.messages.pop();
            }
        self.api_messages = api_messages;
        self.state = AppState::Idle;
        self.status_text.clear();
    }

    // =============================================
    // Plan-and-Execute tracking
    // =============================================

    /// Parse plan steps from assistant text.
    /// Detects lines matching patterns like "1. description" or "- description".
    pub fn detect_plan(&mut self, text: &str) {
        if self.is_processing() {
            self.plan_steps.clear();
            // Only keep lines that look like numbered plan steps
            for line in text.lines() {
                let trimmed = line.trim();
                // Match "N. description" patterns
                if let Some(rest) = trimmed
                    .strip_prefix(|c: char| c.is_ascii_digit())
                    .and_then(|s| s.strip_prefix(". "))
                {
                    let clean = rest.trim_end_matches(['.', '，', ',']);
                    if !clean.is_empty() {
                        self.plan_steps.push(PlanStep {
                            description: clean.to_string(),
                            done: false,
                        });
                    }
                }
            }
        }
    }

    /// Mark the next incomplete plan step as done.
    pub fn mark_next_plan_step_done(&mut self) {
        for step in &mut self.plan_steps {
            if !step.done {
                step.done = true;
                break;
            }
        }
    }

    /// Reset app for a new session (clear messages, etc.)
    pub fn reset_for_new_session(&mut self) {
        self.messages.clear();
        self.message_timestamps.clear();
        self.api_messages = None;
        self.tool_call_count = 0;
        self.status_text.clear();
        self.token_usage = None;
        self.input = InputState::new();
        self.overlay.show_sidebar = false;
        self.http_logs.clear();
        self.overlay.sidebar_selected = 0;
        self.overlay.sidebar_body_idx = None;
        self.overlay.sidebar_body_scroll = 0;
        self.plan_steps.clear();
        self.overlay.session_search.clear();
        self.overlay.session_search_mode = false;
        self.overlay.selected_message = None;
        self.overlay.selection_mode = false;
        self.overlay.tool_call_expanded.clear();
    }

    /// Ensure message_timestamps is in sync with messages after loading from session
    pub fn sync_message_timestamps(&mut self) {
        let now = chrono::Local::now().naive_local();
        while self.message_timestamps.len() < self.messages.len() {
            self.message_timestamps.push(now);
        }
        if self.message_timestamps.len() > self.messages.len() {
            self.message_timestamps.truncate(self.messages.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        let mut c = Config::new();
        c.api_key = "test".to_string();
        c
    }

    #[test]
    fn test_app_new() {
        let app = App::new(test_config());
        assert!(app.messages.is_empty());
        assert_eq!(app.state, AppState::Idle);
        assert!(!app.is_processing());
        assert!(app.http_logs.is_empty());
        assert_eq!(app.current_agent, "default");
        assert_eq!(app.tool_call_count, 0);
    }

    #[test]
    fn test_add_user_message_sets_processing() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], Message::User { ref text } if text == "hello"));
        assert!(app.is_processing());
        assert_eq!(app.scroll_lines, 0);
    }

    #[test]
    fn test_finish_processing_clears_state() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert!(app.is_processing());

        app.finish_processing(Some(vec![serde_json::json!({"role": "assistant", "content": "hi"})]));
        assert!(!app.is_processing());
        assert_eq!(app.state, AppState::Idle);
        assert!(app.status_text.is_empty());
        assert!(app.api_messages.is_some());
    }

    #[test]
    fn test_finish_processing_removes_empty_assistant() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();  // creates empty assistant
        assert_eq!(app.messages.len(), 2);
        app.finish_processing(None);
        assert_eq!(app.messages.len(), 1);  // empty assistant removed
        assert!(!app.is_processing());
    }

    #[test]
    fn test_add_error_sets_idle() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        assert!(app.is_processing());

        app.add_error("something went wrong");
        assert!(!app.is_processing());
        assert_eq!(app.messages.len(), 2);  // user + error
        assert!(matches!(app.messages[1], Message::Error { ref text } if text == "something went wrong"));
        assert!(app.api_messages.is_none());
    }

    #[test]
    fn test_add_error_removes_empty_assistant() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.start_assistant_message();
        assert_eq!(app.messages.len(), 2);
        app.add_error("err");
        assert_eq!(app.messages.len(), 2);  // user + error
        assert!(matches!(app.messages[1], Message::Error { .. }));
    }

    #[test]
    fn test_assistant_message_append() {
        let mut app = App::new(test_config());
        app.start_assistant_message();
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], Message::Assistant { ref text, .. } if text.is_empty()));

        app.append_assistant_text("hello ");
        app.append_assistant_text("world");
        assert!(matches!(app.messages[0], Message::Assistant { ref text, .. } if text == "hello world"));
    }

    #[test]
    fn test_append_assistant_reuses_empty() {
        let mut app = App::new(test_config());
        app.append_assistant_text("direct");
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(app.messages[0], Message::Assistant { ref text, .. } if text == "direct"));
    }

    #[test]
    fn test_add_tool_call() {
        let mut app = App::new(test_config());
        app.add_tool_call("weight", r#"{"action":"list"}"#, "OK", 1, 2);
        assert_eq!(app.messages.len(), 1);
        assert!(matches!(&app.messages[0], Message::ToolCall { name, step: 1, total_steps: 2, .. } if name == "weight"));
        assert_eq!(app.tool_call_count, 1);
    }

    #[test]
    fn test_scroll() {
        let mut app = App::new(test_config());
        assert_eq!(app.scroll_lines, 0);
        app.scroll_up();
        assert_eq!(app.scroll_lines, 3);
        app.scroll_down();
        assert_eq!(app.scroll_lines, 0);
        app.scroll_down();
        assert_eq!(app.scroll_lines, 0);  // saturating
    }

    #[test]
    fn test_set_status() {
        let mut app = App::new(test_config());
        assert!(app.status_text.is_empty());
        app.set_status("thinking…");
        assert_eq!(app.status_text, "thinking…");
    }

    #[test]
    fn test_reset_for_new_session() {
        let mut app = App::new(test_config());
        app.add_user_message("hello");
        app.add_tool_call("test", "{}", "ok", 1, 1);
        app.set_status("done");
        app.reset_for_new_session();
        assert!(app.messages.is_empty());
        assert!(app.message_timestamps.is_empty());
        assert!(app.api_messages.is_none());
        assert_eq!(app.tool_call_count, 0);
        assert!(app.status_text.is_empty());
        assert!(app.http_logs.is_empty());
        assert!(app.plan_steps.is_empty());
    }

    #[test]
    fn test_detect_plan() {
        let mut app = App::new(test_config());
        app.add_user_message("plan something");
        app.detect_plan("1. first step.\n2. second step，\n3. third step.");
        assert_eq!(app.plan_steps.len(), 3);
        assert_eq!(app.plan_steps[0].description, "first step");
        assert!(!app.plan_steps[0].done);
        assert_eq!(app.plan_steps[2].description, "third step");
    }

    #[test]
    fn test_detect_plan_ignores_non_numbered() {
        let mut app = App::new(test_config());
        app.add_user_message("hi");
        app.detect_plan("- bullet item\nplain text\n1. actual step");
        assert_eq!(app.plan_steps.len(), 1);
        assert_eq!(app.plan_steps[0].description, "actual step");
    }

    #[test]
    fn test_detect_plan_only_when_processing() {
        let mut app = App::new(test_config());
        app.detect_plan("1. first step");
        assert!(app.plan_steps.is_empty());  // not processing
    }

    #[test]
    fn test_mark_next_plan_step_done() {
        let mut app = App::new(test_config());
        app.add_user_message("do it");
        app.detect_plan("1. step A\n2. step B");
        app.mark_next_plan_step_done();
        assert!(app.plan_steps[0].done);
        assert!(!app.plan_steps[1].done);
        app.mark_next_plan_step_done();
        assert!(app.plan_steps[1].done);
    }

    #[test]
    fn test_sync_message_timestamps_fills_gaps() {
        let mut app = App::new(test_config());
        app.messages.push(Message::User { text: "a".to_string() });
        app.messages.push(Message::User { text: "b".to_string() });
        assert!(app.message_timestamps.is_empty());
        app.sync_message_timestamps();
        assert_eq!(app.message_timestamps.len(), 2);
    }

    #[test]
    fn test_sync_message_timestamps_truncates_excess() {
        let mut app = App::new(test_config());
        app.messages.push(Message::User { text: "a".to_string() });
        app.message_timestamps.push(chrono::Local::now().naive_local());
        app.message_timestamps.push(chrono::Local::now().naive_local());
        app.sync_message_timestamps();
        assert_eq!(app.message_timestamps.len(), 1);
    }

    #[test]
    fn test_add_http_log_maintains_bound() {
        let mut app = App::new(test_config());
        for i in 0..55 {
            app.add_http_log(HttpLog {
                timestamp: "t".to_string(),
                status: 200,
                duration_ms: i,
                model: "m".to_string(),
                prompt_tokens: 0,
                completion_tokens: 0,
                error: None,
                request_body: String::new(),
            });
        }
        assert_eq!(app.http_logs.len(), 50);
        assert_eq!(app.http_logs[0].duration_ms, 54);  // newest first
    }

    #[test]
    fn test_input_state_insert_and_delete() {
        let mut input = InputState::new();
        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.text, "hi");
        assert_eq!(input.cursor, 2);

        input.move_cursor_left();
        assert_eq!(input.cursor, 1);
        input.insert_char('x');
        assert_eq!(input.text, "hxi");

        input.delete_before_cursor();
        assert_eq!(input.text, "hi");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn test_input_cursor_bounds() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.move_cursor_left();
        assert_eq!(input.cursor, 0);
        input.move_cursor_left();  // no-op
        assert_eq!(input.cursor, 0);

        input.move_cursor_right();
        assert_eq!(input.cursor, 1);
        input.move_cursor_right();  // no-op
        assert_eq!(input.cursor, 1);

        input.delete_before_cursor();  // cursor at 1, should delete 'a'
        assert_eq!(input.text, "");
        input.delete_before_cursor();  // no-op (cursor at 0)
        assert_eq!(input.text, "");
    }

    #[test]
    fn test_input_history() {
        let mut input = InputState::new();
        assert!(input.navigate_up().is_none());  // empty

        input.commit_to_history("hello");
        input.commit_to_history("world");
        assert_eq!(input.history.len(), 2);

        assert_eq!(input.navigate_up().as_deref(), Some("world"));
        assert_eq!(input.navigate_up().as_deref(), Some("hello"));
        assert_eq!(input.navigate_up(), None);  // at start

        assert_eq!(input.navigate_down().as_deref(), Some("world"));
        assert_eq!(input.navigate_down(), None);  // at end (clear)
    }

    #[test]
    fn test_input_history_dedup() {
        let mut input = InputState::new();
        input.commit_to_history("same");
        input.commit_to_history("same");  // duplicate, ignored
        assert_eq!(input.history.len(), 1);
    }

    #[test]
    fn test_input_history_max_50() {
        let mut input = InputState::new();
        for i in 0..60 {
            input.commit_to_history(&format!("item{}", i));
        }
        assert_eq!(input.history.len(), 50);
        assert_eq!(input.history[0], "item10");  // oldest dropped
        assert_eq!(input.history[49], "item59");
    }

    #[test]
    fn test_empty_commit_does_not_add_to_history() {
        let mut input = InputState::new();
        input.commit_to_history("");
        assert!(input.history.is_empty());
    }

    #[test]
    fn test_overlay_filtered_sessions_empty() {
        let overlay = OverlayState::new(vec!["default".to_string()]);
        let result = overlay.filtered_sessions();
        assert!(result.is_empty());
    }

    #[test]
    fn test_overlay_filtered_sessions_search() {
        let mut overlay = OverlayState::new(vec!["default".to_string()]);
        overlay.session_list = vec![
            crate::session::SessionMeta { id: "1".to_string(), title: "Weight tracking".to_string(), agent_id: "default".to_string(), state: crate::session::SessionState::Active, created_at: 0, updated_at: 0, message_count: 0 },
            crate::session::SessionMeta { id: "2".to_string(), title: "Mood log".to_string(), agent_id: "default".to_string(), state: crate::session::SessionState::Active, created_at: 0, updated_at: 0, message_count: 0 },
            crate::session::SessionMeta { id: "3".to_string(), title: "Weight history".to_string(), agent_id: "default".to_string(), state: crate::session::SessionState::Active, created_at: 0, updated_at: 0, message_count: 0 },
        ];

        // No filter -> all
        assert_eq!(overlay.filtered_sessions().len(), 3);

        // With filter
        overlay.session_search = "weight".to_string();
        let result = overlay.filtered_sessions();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|s| s.title.to_lowercase().contains("weight")));

        overlay.session_search = "mood".to_string();
        assert_eq!(overlay.filtered_sessions().len(), 1);

        overlay.session_search = "nonexistent".to_string();
        assert!(overlay.filtered_sessions().is_empty());
    }

    #[test]
    fn test_overlay_filtered_sessions_case_insensitive() {
        let mut overlay = OverlayState::new(vec!["default".to_string()]);
        overlay.session_list = vec![
            crate::session::SessionMeta { id: "1".to_string(), title: "Weight Tracking".to_string(), agent_id: "default".to_string(), state: crate::session::SessionState::Active, created_at: 0, updated_at: 0, message_count: 0 },
        ];
        overlay.session_search = "WEIGHT".to_string();
        assert_eq!(overlay.filtered_sessions().len(), 1);
    }
}
