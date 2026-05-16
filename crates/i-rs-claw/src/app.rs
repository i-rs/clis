use crate::config::Config;
use serde_json::Value;

#[derive(Clone)]
pub enum Message {
    User { text: String },
    Assistant { text: String },
    ToolCall { name: String, args: String, result: String },
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
    pub tool_index_text: String,
    /// Input history for up/down navigation (most recent last)
    pub input_history: Vec<String>,
    /// Current position in input history (None = fresh input)
    pub input_history_index: Option<usize>,
    /// Cursor position within input (byte index)
    pub input_cursor: usize,
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
        }
    }

    pub fn is_processing(&self) -> bool {
        matches!(self.state, AppState::Processing)
    }

    pub fn add_user_message(&mut self, text: &str) {
        self.messages
            .push(Message::User { text: text.to_string() });
        self.state = AppState::Processing;
    }

    // ── Input cursor manipulation ──

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
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

    /// Update the real-time status text (shown in status bar)
    pub fn set_status(&mut self, text: &str) {
        self.status_text = text.to_string();
    }

    /// Start a new assistant message. If the last message is an empty assistant,
    /// reuse it instead of creating a new one.
    pub fn start_assistant_message(&mut self) {
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

    pub fn add_tool_call(&mut self, name: &str, args: &str, result: &str) {
        self.messages.push(Message::ToolCall {
            name: name.to_string(),
            args: args.to_string(),
            result: result.to_string(),
        });
        self.tool_call_count += 1;
    }

    pub fn add_error(&mut self, text: &str) {
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
    }
}
