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
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            state: AppState::Idle,
            config,
            tool_call_count: 0,
            status_text: String::new(),
            api_messages: None,
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
        self.messages
            .push(Message::Error { text: text.to_string() });
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
}
