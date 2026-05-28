use serde_json::Value;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Token(String),
    Reasoning(String),
    ToolCallStart { id: String, name: String, args: Value },
    ToolCallEnd { id: String, name: String, result: String },
    #[allow(dead_code)]
    FileChanged { path: String },
    Status(String),
    Plan { steps: Vec<String> },
    Done { usage: Option<crate::provider::Usage>, messages: Vec<crate::provider::LlmMessage>, context_pct: f64 },
    Error(String),
}
