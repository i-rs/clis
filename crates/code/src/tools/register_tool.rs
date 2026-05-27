use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};
use crate::config::Config;

pub struct RegisterTool;

#[async_trait]
impl Tool for RegisterTool {
    fn name(&self) -> &str { "register_tool" }
    fn description(&self) -> &str { "Register a created tool in the manifest for claw discovery" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "register_tool",
                "description": "Register a created tool so claw can discover and use it",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Tool name (e.g. i-rs-mood)"},
                        "description": {"type": "string", "description": "Tool description"},
                        "commands": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Available commands"
                        },
                        "source_path": {"type": "string", "description": "Path to the crate source"},
                        "binary_path": {"type": "string", "description": "Path to the installed binary"}
                    },
                    "required": ["name", "description", "commands"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("name required"))?;
        let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let commands: Vec<String> = args.get("commands")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let source_path = args.get("source_path").and_then(|v| v.as_str()).unwrap_or("");
        let binary_path = args.get("binary_path").and_then(|v| v.as_str()).unwrap_or("");

        // In agent mode, output tool_created event for claw
        if std::env::var("I_RS_CODE_AGENT_MODE").is_ok() {
            return Ok(json!({
                "requires_registration": true,
                "tool": {
                    "name": name,
                    "description": description,
                    "commands": commands,
                    "source_path": source_path,
                    "binary_path": binary_path
                }
            }).to_string());
        }

        // Standalone mode: update manifest directly
        let config = Config::load()?;
        let manifest_path = config.manifest_path();
        let mut manifest: Value = if manifest_path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?
        } else {
            json!({"tools": []})
        };

        let tool_entry = json!({
            "name": name,
            "version": "0.1.0",
            "description": description,
            "commands": commands,
            "source": source_path,
            "binary": binary_path,
            "created_at": chrono::Utc::now().to_rfc3339()
        });

        if let Some(tools) = manifest["tools"].as_array_mut() {
            // Update if exists, else add
            if let Some(existing) = tools.iter_mut().find(|t| t["name"] == name) {
                *existing = tool_entry;
            } else {
                tools.push(tool_entry);
            }
        }

        if let Some(parent) = manifest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        Ok(format!("Registered tool '{}' in manifest", name))
    }
}
