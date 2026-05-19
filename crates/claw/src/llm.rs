use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug)]
pub enum LlmEvent {
    /// A text token from the streaming response
    Token(String),
    /// Reasoning content from the model (DeepSeek chain-of-thought)
    Reasoning(String),
    /// Signals the app to start a new assistant message (for multi-round responses)
    NewRound,
    /// A tool was executed (with result)
    ToolExecuted {
        name: String,
        args: String,
        result: String,
        /// Zero-based index of this tool call in the current round
        step: usize,
        /// Total number of tool calls in the current round
        total_steps: usize,
    },
    /// Real-time status update (shown in status bar)
    Status(String),
    /// An error occurred
    Error(String),
    /// All responses complete, carries final API message list and optional token usage
    Done(Vec<Value>, Option<TokenUsage>),
    /// HTTP request log for debug sidebar
    HttpLog {
        status: u16,
        duration_ms: u64,
        model: String,
        prompt_tokens: u32,
        completion_tokens: u32,
        error: Option<String>,
        request_body: String,
    },
}

#[derive(Default, Clone)]
pub(crate) struct ToolCallAcc {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) arguments: String,
}

pub(crate) enum StreamResult {
    Text(Option<TokenUsage>, String), // usage + accumulated text content
    ToolCalls(Vec<(ToolCallAcc, Value)>, String), // tool_calls + accumulated reasoning_content
}
