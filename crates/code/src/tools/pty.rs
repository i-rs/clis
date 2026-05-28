use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::{json, Map, Value};

use crate::pty::PtyManager;
use crate::tools::{Tool, ToolResult};

static PTY_MANAGER: Lazy<PtyManager> = Lazy::new(PtyManager::new);

pub struct PtyExecTool;
pub struct PtyInterruptTool;

#[async_trait]
impl Tool for PtyExecTool {
    fn name(&self) -> &str { "pty_exec" }
    fn description(&self) -> &str {
        "Execute a command in a persistent shell session. Use for interactive/long-running commands. The session is auto-created and reused per session_id."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "pty_exec",
                "description": "Execute a command in a persistent shell session. Keeps shell state (cd, env, vars) between calls.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Unique ID to reuse or create the session"
                        },
                        "command": {
                            "type": "string",
                            "description": "Shell command to execute"
                        },
                        "timeout_secs": {
                            "type": "integer",
                            "description": "Max execution time in seconds (default 120)",
                            "default": 120
                        }
                    },
                    "required": ["session_id", "command"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let session_id = args.get("session_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("session_id required"))?;
        let command = args.get("command").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("command required"))?;
        let timeout = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(120);
        let cwd = std::env::current_dir().unwrap_or_default();
        let output = PTY_MANAGER.exec(session_id, command, timeout, &cwd).await?;
        Ok(output)
    }
}

#[async_trait]
impl Tool for PtyInterruptTool {
    fn name(&self) -> &str { "pty_interrupt" }
    fn description(&self) -> &str { "Kill the process running in a PTY session." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "pty_interrupt",
                "description": "Kill the running process in a PTY session.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "PTY session ID to interrupt"
                        }
                    },
                    "required": ["session_id"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let session_id = args.get("session_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("session_id required"))?;
        PTY_MANAGER.interrupt(session_id).await?;
        Ok(format!("Interrupted session {}", session_id))
    }
}
