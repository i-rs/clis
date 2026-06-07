pub mod loader;
pub mod verifier;

use std::collections::HashMap;

/// Information about a tool call made by the agent.
#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub tool: String,
    pub command: Option<String>,
    pub args: HashMap<String, String>,
}

impl ToolCallInfo {
    pub fn new(tool: &str) -> Self {
        Self {
            tool: tool.to_string(),
            command: None,
            args: HashMap::new(),
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    pub fn with_arg(mut self, key: &str, val: &str) -> Self {
        self.args.insert(key.to_string(), val.to_string());
        self
    }
}

/// Metadata for a conversation script.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptMeta {
    pub tool: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub storage_backends: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// One step in a conversation script.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptStep {
    pub step: u32,
    pub title: String,
    pub user_message: String,
    #[serde(default)]
    pub expected_tool: Option<String>,
    #[serde(default)]
    pub expected_command: Option<String>,
    #[serde(default)]
    pub expected_args: Option<HashMap<String, String>>,
    #[serde(default)]
    pub expected_flags: Option<HashMap<String, String>>,
    #[serde(default)]
    pub check_reply: Option<ReplyCheck>,
    #[serde(default)]
    pub verify_storage: Option<StorageCheck>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplyCheck {
    #[serde(default)]
    pub contains: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageCheck {
    #[serde(default)]
    pub file_check: Option<String>,
    #[serde(default)]
    pub expected_state: Option<serde_json::Value>,
    #[serde(default)]
    pub no_new_records: Option<bool>,
}

/// A complete conversation script loaded from disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Script {
    pub meta: ScriptMeta,
    pub steps: Vec<ScriptStep>,
}
