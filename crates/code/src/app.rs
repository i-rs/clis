use crate::config::Config;
use crate::tui::input::InputState;
use std::collections::HashSet;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "role")]
pub enum AgentMessage {
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "assistant")]
    Assistant { content: String, reasoning: String, tool_calls: Option<Vec<serde_json::Value>>, #[serde(default)] reasoning_expanded: bool },
    #[serde(rename = "tool")]
    ToolResult { content: String },
    #[serde(rename = "system")]
    System { content: String },
    #[serde(rename = "file_edit")]
    FileEdit { path: String, summary: String },
    #[serde(rename = "separator")]
    Separator { label: String },
}

impl AgentMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self::User { content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::Assistant { content: content.into(), reasoning: String::new(), tool_calls: None, reasoning_expanded: false }
    }
    pub fn tool(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self::ToolResult { content: format!("{}\n{}", name.into(), content.into()) }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self::System { content: content.into() }
    }
}

#[derive(Clone, Default)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
}

pub enum AppMode {
    Idle,
    Waiting,
}

#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub name: String,
    pub args: String,
    pub result: Option<String>,
}

pub struct StreamingState {
    pub content: String,
    pub reasoning: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub current_tool: Option<ToolCallInfo>,
}

pub struct App {
    pub config: Config,
    pub input: InputState,
    pub messages: Vec<AgentMessage>,
    pub agent_messages: Vec<crate::provider::LlmMessage>,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
    pub file_changes: HashSet<String>,
    pub token_usage: TokenUsage,
    pub version: String,
    pub current_dir: String,
    pub mode: AppMode,
    pub streaming: Option<StreamingState>,
    pub session_id: Option<String>,
    pub cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
    pub show_shortcuts: bool,
    pub show_debug: bool,
    pub debug_scroll: usize,
    pub should_quit: bool,
    pub tool_names: Vec<String>,
    pub last_file_states: Vec<(String, String)>,
    pub context_usage: Option<f64>,
    pub status_message: Option<String>,
    pub show_transcript: bool,
    pub transcript_scroll: usize,
}

impl App {
    pub fn new(config: Config, session_id: Option<String>) -> Self {
        let current_dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".into());

        Self {
            config,
            input: InputState::new(),
            messages: Vec::new(),
            agent_messages: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
            file_changes: HashSet::new(),
            token_usage: TokenUsage::default(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            current_dir,
            mode: AppMode::Idle,
            streaming: None,
            session_id,
            cancel_tx: None,
            task_handle: None,
            show_shortcuts: false,
            show_debug: false,
            debug_scroll: 0,
            should_quit: false,
            tool_names: Vec::new(),
            last_file_states: Vec::new(),
            context_usage: None,
            status_message: None,
            show_transcript: false,
            transcript_scroll: 0,
        }
    }

    pub fn start_streaming(&mut self) {
        self.streaming = Some(StreamingState {
            content: String::new(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
            current_tool: None,
        });
    }

    pub fn push_token(&mut self, token: &str) {
        if let Some(ref mut s) = self.streaming {
            s.content.push_str(token);
        }
    }

    pub fn finish_streaming(&mut self) -> (String, String) {
        let s = self.streaming.take();
        match s {
            Some(s) => (s.content, s.reasoning),
            None => (String::new(), String::new()),
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
        self.auto_scroll = false;
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn add_token_usage(&mut self, input: u32, output: u32) {
        self.token_usage.input = self.token_usage.input.saturating_add(input);
        self.token_usage.output = self.token_usage.output.saturating_add(output);
    }
}
