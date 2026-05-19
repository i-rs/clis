use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

/// A built-in tool that allows the LLM to persist user information
/// (name, interests, habits, preferences) to CrossSessionMemory.
///
/// The tool itself only validates and returns a confirmation;
/// actual persistence happens in tui.rs when the ToolExecuted event is handled.
pub struct UserMemoryTool;

impl ClawTool for UserMemoryTool {
    fn name(&self) -> &str {
        "update_user_memory"
    }

    fn description(&self) -> &str {
        "Save information you learned about the user — their name, interests, habits, preferences, or any personal facts. Call this whenever the user shares something about themselves so you can remember it in future conversations."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "user_name": {
                    "type": "string",
                    "description": "用户的名字或称呼"
                },
                "user_info": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "关于用户的事实信息，如兴趣爱好、生活习惯、职业等"
                },
                "preferences": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "用户偏好，如数据管理方式、交互风格偏好等"
                }
            },
            "additionalProperties": false
        })
    }

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let mut saved: Vec<String> = Vec::new();

        if let Some(name) = args
            .get("user_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            saved.push(format!("称呼: {}", name));
        }

        if let Some(info) = args.get("user_info").and_then(|v| v.as_array()) {
            for item in info {
                if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                    saved.push(format!("用户信息: {}", s));
                }
            }
        }

        if let Some(prefs) = args.get("preferences").and_then(|v| v.as_array()) {
            for item in prefs {
                if let Some(s) = item.as_str().filter(|s| !s.is_empty()) {
                    saved.push(format!("偏好: {}", s));
                }
            }
        }

        if saved.is_empty() {
            return Err("没有需要保存的用户信息".to_string());
        }

        Ok(format!("已保存用户信息:\n{}", saved.join("\n")))
    }
}
