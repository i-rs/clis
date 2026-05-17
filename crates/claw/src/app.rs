use crate::config::Config;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    Assistant { text: String },
    ToolCall {
        name: String,
        args: String,
        result: String,
        step: usize,
        total_steps: usize,
    },
    Error { text: String },
}

#[derive(Clone, PartialEq)]
pub enum AppState {
    Idle,
    Processing,
}

pub struct App {
    pub messages: Vec<Message>,
    pub input: String,
    pub state: AppState,
    pub config: Config,
    pub tool_call_count: usize,
    /// Real-time status text shown in status bar (e.g. "思考中…", "正在调用工具…")
    pub status_text: String,
    /// Full API message list preserved across turns (includes tool call context)
    pub api_messages: Option<Vec<Value>>,
    /// Whether the session list overlay is shown
    pub show_session_list: bool,
    /// Currently selected index in session list
    pub session_list_index: usize,
    /// Cached session list for display
    pub session_list: Vec<crate::session::SessionMeta>,
    /// Token usage from the last LLM response
    pub token_usage: Option<crate::llm::TokenUsage>,
    /// Filtered tool index text (respects enabled_tools)
    #[allow(dead_code)]
    pub tool_index_text: String,
    /// Input history for up/down navigation (most recent last)
    pub input_history: Vec<String>,
    /// Current position in input history (None = fresh input)
    pub input_history_index: Option<usize>,
    /// Cursor position within input (byte index)
    pub input_cursor: usize,
    /// How many lines the user has scrolled up from the bottom (0 = bottom)
    pub scroll_offset: usize,
    /// Whether the HTTP debug sidebar is shown
    pub show_sidebar: bool,
    /// HTTP request logs (newest first)
    pub http_logs: Vec<HttpLog>,
    /// Selected index in the sidebar
    pub sidebar_selected: usize,
    /// If set, shows the full request body for this log entry
    pub sidebar_body_idx: Option<usize>,
    /// Scroll offset within the body overlay
    pub sidebar_body_scroll: usize,
    /// Current reasoning text from LLM (DeepSeek chain-of-thought)
    pub current_reasoning: String,
    /// Proactive reminder text from i-rs remind (shown to LLM on next user message)
    pub reminder_text: Option<String>,
    /// Current execution plan steps (for plan-and-execute)
    pub plan_steps: Vec<PlanStep>,
    /// Session list search/filter text
    pub session_search: String,
    /// Whether session search mode is active
    pub session_search_mode: bool,
    /// Tab completion candidates for the current input
    pub tab_completions: Vec<String>,
    /// Current index in tab completion cycle
    pub tab_completion_index: usize,
    /// Rename buffer when renaming a session
    pub session_rename_buf: String,
    /// Whether delete confirmation is shown
    pub session_confirm_delete: bool,
    /// Transient feedback text (e.g. "已复制"), cleared on next user interaction
    pub copy_feedback: Option<String>,
    /// Current agent profile ID
    pub current_agent: String,
}

impl App {
    pub fn new(config: Config) -> Self {
        let enabled = if config.enabled_tools.is_empty() {
            None
        } else {
            Some(&config.enabled_tools)
        };
        let tool_index_text = crate::tools::format_index(enabled);

        Self {
            messages: Vec::new(),
            input: String::new(),
            input_cursor: 0,
            state: AppState::Idle,
            config,
            tool_call_count: 0,
            status_text: String::new(),
            api_messages: None,
            show_session_list: false,
            session_list_index: 0,
            session_list: Vec::new(),
            token_usage: None,
            tool_index_text,
            input_history: Vec::new(),
            input_history_index: None,
            scroll_offset: 0,
            show_sidebar: false,
            http_logs: Vec::new(),
            sidebar_selected: 0,
            sidebar_body_idx: None,
            sidebar_body_scroll: 0,
            current_reasoning: String::new(),
            reminder_text: None,
            plan_steps: Vec::new(),
            session_search: String::new(),
            session_search_mode: false,
            tab_completions: Vec::new(),
            tab_completion_index: 0,
            session_rename_buf: String::new(),
            session_confirm_delete: false,
            copy_feedback: None,
            current_agent: "default".to_string(),
        }
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.state, AppState::Processing)
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.copy_feedback.take();
        self.messages
            .push(Message::User { text: text.to_string() });
        self.state = AppState::Processing;
        self.scroll_offset = 0;
        self.plan_steps.clear(); // Clear plan from previous turn
    }

    // ── Input cursor manipulation ──

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.copy_feedback.take();
        self.input.insert(self.input_cursor, c);
        self.input_cursor += c.len_utf8();
    }

    /// Delete the character before the cursor (Backspace).
    pub fn delete_before_cursor(&mut self) {
        if self.input_cursor == 0 {
            return;
        }
        let prev = self.input[..self.input_cursor].char_indices().next_back();
        if let Some((idx, _)) = prev {
            self.input.drain(idx..self.input_cursor);
            self.input_cursor = idx;
        }
    }

    /// Delete the character at the cursor (Delete key).
    #[allow(dead_code)]
    pub fn delete_at_cursor(&mut self) {
        if self.input_cursor >= self.input.len() {
            return;
        }
        let next = self.input[self.input_cursor..].char_indices().nth(1);
        let end = match next {
            Some((offset, _)) => self.input_cursor + offset,
            None => self.input.len(),
        };
        self.input.drain(self.input_cursor..end);
    }

    pub fn move_cursor_left(&mut self) {
        if self.input_cursor == 0 {
            return;
        }
        let prev = self.input[..self.input_cursor].char_indices().next_back();
        if let Some((idx, _)) = prev {
            self.input_cursor = idx;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.input_cursor >= self.input.len() {
            return;
        }
        let next = self.input[self.input_cursor..].char_indices().nth(1);
        if let Some((offset, _)) = next {
            self.input_cursor += offset;
        } else {
            self.input_cursor = self.input.len();
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.input_cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.input_cursor = self.input.len();
    }

    /// Push text into input history (max 50 entries), reset history index.
    pub fn commit_input_to_history(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        // Avoid duplicating consecutive identical inputs
        if self.input_history.last().map(|s| s.as_str()) != Some(text) {
            self.input_history.push(text.to_string());
            if self.input_history.len() > 50 {
                self.input_history.remove(0);
            }
        }
        self.input_history_index = None;
        self.scroll_offset = 0;
    }

    /// Navigate up in input history: restore previous input.
    /// Returns the text to put in `input`, or None if already at start.
    pub fn navigate_history_up(&mut self) -> Option<String> {
        if self.input_history.is_empty() {
            return None;
        }
        let new_idx = match self.input_history_index {
            Some(i) if i > 0 => i - 1,
            None => self.input_history.len().saturating_sub(1),
            _ => return None,
        };
        self.input_history_index = Some(new_idx);
        Some(self.input_history[new_idx].clone())
    }

    /// Navigate down in input history: go to next input, or clear if at end.
    /// Returns Some(text) to put in `input`, or None to clear.
    pub fn navigate_history_down(&mut self) -> Option<String> {
        match self.input_history_index {
            Some(i) if i + 1 < self.input_history.len() => {
                self.input_history_index = Some(i + 1);
                Some(self.input_history[i + 1].clone())
            }
            Some(_) => {
                // Reached end of history, clear
                self.input_history_index = None;
                None
            }
            None => None,
        }
    }

    // =============================================
    // Message scroll
    // =============================================

    /// Scroll messages up (toward older messages).
    pub fn scroll_up(&mut self) {
        if self.scroll_offset < self.messages.len() {
            self.scroll_offset += 1;
        }
    }

    /// Scroll messages down (toward newer messages).
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Update the real-time status text (shown in status bar)
    pub fn set_status(&mut self, text: &str) {
        self.status_text = text.to_string();
    }

    /// Start a new assistant message. If the last message is an empty assistant,
    /// reuse it instead of creating a new one.
    pub fn start_assistant_message(&mut self) {
        self.current_reasoning.clear();
        let is_empty_assistant = matches!(
            self.messages.last(),
            Some(Message::Assistant { text }) if text.is_empty()
        );
        if !is_empty_assistant {
            self.messages
                .push(Message::Assistant { text: String::new() });
        }
    }

    pub fn append_assistant_text(&mut self, text: &str) {
        let last_is_assistant =
            matches!(self.messages.last_mut(), Some(Message::Assistant { .. }));
        if !last_is_assistant {
            self.start_assistant_message();
        }
        if let Some(Message::Assistant { text: t }) = self.messages.last_mut() {
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
        self.tool_call_count += 1;
    }

    pub fn add_http_log(&mut self, log: HttpLog) {
        self.http_logs.insert(0, log);
        if self.http_logs.len() > 50 {
            self.http_logs.pop();
        }
    }

    pub fn add_error(&mut self, text: &str) {
        self.current_reasoning.clear();
        // Remove trailing empty assistant message (from NewRound before error)
        if let Some(Message::Assistant { text }) = self.messages.last() {
            if text.is_empty() {
                self.messages.pop();
            }
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
        self.current_reasoning.clear();
        // Remove trailing empty assistant message
        if let Some(Message::Assistant { text }) = self.messages.last() {
            if text.is_empty() {
                self.messages.pop();
            }
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
                    let clean = rest.trim_end_matches(|c: char| c == '.' || c == '，' || c == ',');
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
        self.api_messages = None;
        self.tool_call_count = 0;
        self.status_text.clear();
        self.token_usage = None;
        self.input.clear();
        self.input_cursor = 0;
        self.input_history.clear();
        self.input_history_index = None;
        self.show_sidebar = false;
        self.http_logs.clear();
        self.sidebar_selected = 0;
        self.sidebar_body_idx = None;
        self.sidebar_body_scroll = 0;
        self.plan_steps.clear();
        self.session_search.clear();
        self.session_search_mode = false;
    }
}
