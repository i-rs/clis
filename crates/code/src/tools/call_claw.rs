use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct CallClawTool;

#[async_trait]
impl Tool for CallClawTool {
    fn name(&self) -> &str { "call_claw" }
    fn description(&self) -> &str { "Send a request to claw for help (debug, review, approval)" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "call_claw",
                "description": "Request help from claw. Use when stuck on build errors, need design review, or need user input.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "request_type": {
                            "type": "string",
                            "enum": ["debug", "review", "approval", "info"],
                            "description": "Type of request"
                        },
                        "content": {"type": "string", "description": "The question or problem description"},
                        "context": {"type": "object", "description": "Additional context (file paths, errors, etc.)"}
                    },
                    "required": ["request_type", "content"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let request_type = args.get("request_type").and_then(|v| v.as_str()).unwrap_or("info");
        let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");

        // In agent mode, send to claw via protocol module.
        // In standalone mode, return a message asking the user.
        if std::env::var("I_RS_CODE_AGENT_MODE").is_ok() {
            // Protocol layer handles this - but we need to signal from here.
            // The agent loop will detect this response and create the protocol event.
            Ok(json!({
                "requires_claw": true,
                "request_type": request_type,
                "content": content
            }).to_string())
        } else {
            Ok(format!("[Need input from you]: {}\n\n(Type your response or guidance above)", content))
        }
    }
}
