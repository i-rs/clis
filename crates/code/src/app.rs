use crate::config::Config;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self {
            input: 0,
            output: 0,
        }
    }
}

pub enum AppMode {
    Idle,
    Waiting,
    ConfirmQuit,
}

#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub name: String,
    pub args: String,
    pub result: Option<String>,
}

pub struct StreamingState {
    pub content: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub current_tool: Option<ToolCallInfo>,
}

pub struct App {
    pub config: Config,
    pub input: String,
    pub cursor_pos: usize,
    pub messages: Vec<ChatMessage>,
    pub scroll_offset: usize,
    pub file_changes: HashSet<String>,
    pub token_usage: TokenUsage,
    pub version: String,
    pub current_dir: String,
    pub mode: AppMode,
    pub streaming: Option<StreamingState>,
    pub session_id: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(config: Config, session_id: Option<String>) -> Self {
        let current_dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".into());

        Self {
            config,
            input: String::new(),
            cursor_pos: 0,
            messages: Vec::new(),
            scroll_offset: 0,
            file_changes: HashSet::new(),
            token_usage: TokenUsage::default(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            current_dir,
            mode: AppMode::Idle,
            streaming: None,
            session_id,
            should_quit: false,
        }
    }

    pub fn start_streaming(&mut self) {
        self.streaming = Some(StreamingState {
            content: String::new(),
            tool_calls: Vec::new(),
            current_tool: None,
        });
    }

    pub fn push_token(&mut self, token: &str) {
        if let Some(ref mut s) = self.streaming {
            s.content.push_str(token);
        }
    }

    pub fn finish_streaming(&mut self) -> String {
        let content = self.streaming.take().map(|s| s.content).unwrap_or_default();
        content
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let len = self.input[..self.cursor_pos]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_pos -= len;
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            let len = self.input[..self.cursor_pos]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_pos -= len;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            let len = self.input[self.cursor_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.cursor_pos += len;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.input.len();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn add_token_usage(&mut self, input: u32, output: u32) {
        self.token_usage.input = self.token_usage.input.saturating_add(input);
        self.token_usage.output = self.token_usage.output.saturating_add(output);
    }
}
