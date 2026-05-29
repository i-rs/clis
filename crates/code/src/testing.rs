//! 测试基础设施：MockLlmProvider、MockTool、辅助函数

use crate::provider::*;
use crate::tools::{Tool, ToolResult, ToolRegistry};
use async_trait::async_trait;
use serde_json::{json, Value, Map};
use std::sync::Arc;

// ---- Mock Provider ----

pub struct MockLlmProvider {
    events: Vec<StreamEventKind>,
    name: String,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self { events: Vec::new(), name: "mock".into() }
    }

    pub fn with_token(mut self, token: &str) -> Self {
        self.events.push(StreamEventKind::Token(token.to_string()));
        self
    }

    pub fn with_reasoning(mut self, text: &str) -> Self {
        self.events.push(StreamEventKind::Reasoning(text.to_string()));
        self
    }

    pub fn with_tool_call(mut self, name: &str, id: &str, args_json: &str) -> Self {
        let args: Value = serde_json::from_str(args_json).unwrap_or_default();
        self.events.push(StreamEventKind::ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            args,
        });
        self
    }

    pub fn with_response(text: &str) -> Self {
        Self {
            events: vec![
                StreamEventKind::Token(text.to_string()),
                StreamEventKind::Done { usage: None },
            ],
            name: "mock".into(),
        }
    }

    pub fn with_tool_only(name: &str, id: &str, args_json: &str) -> Self {
        let args: Value = serde_json::from_str(args_json).unwrap_or_default();
        Self {
            events: vec![
                StreamEventKind::ToolCall { id: id.to_string(), name: name.to_string(), args },
                StreamEventKind::Done { usage: None },
            ],
            name: "mock".into(),
        }
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.events.push(StreamEventKind::Error(error.to_string()));
        self
    }

    pub fn with_text_and_tool(text: &str, name: &str, id: &str, args_json: &str) -> Self {
        let args: Value = serde_json::from_str(args_json).unwrap_or_default();
        Self {
            events: vec![
                StreamEventKind::Token(text.to_string()),
                StreamEventKind::ToolCall { id: id.to_string(), name: name.to_string(), args },
                StreamEventKind::Done { usage: None },
            ],
            name: "mock".into(),
        }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    fn name(&self) -> &str { &self.name }

    async fn stream(&self, _messages: &[LlmMessage], _tool_defs: &[Value]) -> StreamRx {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        let events = self.events.clone();
        tokio::spawn(async move {
            for event in events {
                if tx.send(StreamEvent { kind: event }).await.is_err() {
                    break;
                }
            }
        });
        rx
    }

    async fn chat(&self, _messages: &[LlmMessage], _tool_defs: &[Value]) -> anyhow::Result<LlmResponse> {
        Ok(LlmResponse {
            content: None,
            reasoning: String::new(),
            tool_calls: Vec::new(),
            usage: None,
        })
    }
}

// ---- Mock Tool ----

pub struct MockTool {
    name: String,
    response: Result<String, String>,
}

impl MockTool {
    pub fn new(name: &str, response: &str) -> Self {
        Self { name: name.to_string(), response: Ok(response.to_string()) }
    }

    pub fn with_error(name: &str, error: &str) -> Self {
        Self { name: name.to_string(), response: Err(error.to_string()) }
    }
}

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { "Mock tool for testing" }
    fn schema(&self) -> Value {
        json!({"type": "function", "function": {"name": self.name, "description": "mock", "parameters": {"type": "object", "properties": {}}}})
    }
    async fn call(&self, _args: &Map<String, Value>) -> ToolResult {
        match &self.response {
            Ok(s) => Ok(s.clone()),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }
}

/// Create a ToolRegistry with mock tools for testing
pub fn mock_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new_empty();
    registry.register(Arc::new(MockTool::new("read", "file content line 1\nline 2\n")));
    registry.register(Arc::new(MockTool::new("write", "Created test.txt (10 bytes)")));
    registry.register(Arc::new(MockTool::new("bash", "stdout:\nhello")));
    registry.register(Arc::new(MockTool::new("grep", "Found 2 matches:\nmain.rs:10:fn main()\nlib.rs:5:fn main()")));
    registry
}

/// Build a minimal config for testing
pub fn test_config() -> crate::config::Config {
    crate::config::Config {
        provider: "mock".into(),
        api_key: Some("test-key".into()),
        base_url: None,
        model: None,
        workspace: None,
        tools_dir: None,
        bin_dir: None,
        max_rounds: 5,
        max_tool_retries: 2,
        tool_timeout_secs: 30,
        agents: std::collections::HashMap::new(),
        mcp_servers: Vec::new(),
        search_provider: None,
        search_api_key: None,
        max_cost_per_session: None,
    }
}
