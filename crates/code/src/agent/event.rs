use serde_json::Value;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Token(String),
    ToolCallStart { id: String, name: String, args: Value },
    ToolCallEnd { id: String, name: String, result: String },
    Done { usage: Option<crate::provider::Usage>, messages: Vec<crate::provider::LlmMessage> },
    Error(String),
}
