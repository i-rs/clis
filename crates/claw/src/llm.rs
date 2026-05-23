use crate::stats::TokenRecord;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

/// HTTP request/response log data for the debug sidebar.
#[derive(Debug, Clone)]
pub struct HttpLogData {
    pub status: u16,
    pub duration_ms: u64,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub error: Option<String>,
    pub request_body: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
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
    Done(Arc<Vec<Value>>, Option<TokenUsage>),
    /// HTTP request log for debug sidebar
    HttpLog(HttpLogData),
    /// Token usage record for statistics persistence
    UsageRecord(TokenRecord),
}

#[derive(Default, Clone)]
pub struct ToolCallAcc {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Clone)]
pub(crate) enum StreamResult {
    Text(Option<TokenUsage>, String, String), // usage + accumulated text content + accumulated reasoning content
    ToolCalls(Vec<(ToolCallAcc, Value)>, String), // tool_calls + accumulated reasoning_content
}
