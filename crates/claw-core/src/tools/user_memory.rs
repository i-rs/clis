use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

/// A built-in tool that allows the LLM to persist user information
/// (name, interests, habits, preferences) to CrossSessionMemory.
///
/// Architecture: this tool validates and formats user info, then returns a
/// confirmation. Actual persistence is handled centrally by
/// [`crate::core::record_tool_memory`] when the ToolExecuted event fires.
/// This centralized design ensures all execution paths (TUI, Dashboard,
/// Gateway) share the same persistence logic — no path can forget to flush.
pub struct UserMemoryTool;

#[async_trait::async_trait]
impl ClawTool for UserMemoryTool {
    fn name(&self) -> &str {
        "update_user_memory"
    }

    fn description(&self) -> &str {
        "Save information you learned about the user — their name, interests, habits, preferences, or what they call you. Call this whenever the user shares something about themselves so you can remember it in future conversations."
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
                },
                "assistant_nickname": {
                    "type": "string",
                    "description": "用户给你起的昵称或称呼，保存下来以便在对话中使用"
                }
            },
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
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

        if let Some(nick) = args
            .get("assistant_nickname")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            saved.push(format!("你的称呼: {}", nick));
        }

        if saved.is_empty() {
            return Err(ClawError::Validation("没有需要保存的用户信息".to_string()));
        }

        Ok(format!("已保存用户信息:\n{}", saved.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ClawTool;
    use serde_json::json;

    fn test_ctx() -> ToolContext {
        ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: reqwest::Client::new(),
            delegate_runtime: None,
            user_id: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_save_user_name() {
        let tool = UserMemoryTool;
        let result = tool
            .execute(&json!({"user_name": "Alice"}), &test_ctx())
            .await
            .unwrap();
        assert!(result.contains("Alice"));
        assert!(result.contains("称呼"));
    }

    #[tokio::test]
    async fn test_save_user_info() {
        let tool = UserMemoryTool;
        let result = tool
            .execute(
                &json!({
                    "user_info": ["喜欢跑步", "程序员"]
                }),
                &test_ctx(),
            )
            .await
            .unwrap();
        assert!(result.contains("喜欢跑步"));
        assert!(result.contains("程序员"));
    }

    #[tokio::test]
    async fn test_save_preferences() {
        let tool = UserMemoryTool;
        let result = tool
            .execute(
                &json!({
                    "preferences": ["简洁输出", "中文优先"]
                }),
                &test_ctx(),
            )
            .await
            .unwrap();
        assert!(result.contains("简洁输出"));
    }

    #[tokio::test]
    async fn test_save_assistant_nickname() {
        let tool = UserMemoryTool;
        let result = tool
            .execute(&json!({"assistant_nickname": "小助手"}), &test_ctx())
            .await
            .unwrap();
        assert!(result.contains("小助手"));
    }

    #[tokio::test]
    async fn test_save_all_fields() {
        let tool = UserMemoryTool;
        let result = tool
            .execute(
                &json!({
                    "user_name": "Bob",
                    "user_info": ["设计师"],
                    "preferences": ["暗色模式"],
                    "assistant_nickname": "Bob助手"
                }),
                &test_ctx(),
            )
            .await
            .unwrap();
        assert!(result.contains("Bob"));
        assert!(result.contains("设计师"));
        assert!(result.contains("暗色模式"));
        assert!(result.contains("Bob助手"));
    }

    #[tokio::test]
    async fn test_save_empty_args() {
        let tool = UserMemoryTool;
        let result = tool.execute(&json!({}), &test_ctx()).await;
        assert!(result.is_err(), "empty args should fail");
        assert!(result.unwrap_err().to_string().contains("没有需要保存"));
    }

    #[tokio::test]
    async fn test_save_ignores_empty_strings() {
        let tool = UserMemoryTool;
        let result = tool
            .execute(
                &json!({
                    "user_name": "",
                    "user_info": [""],
                    "preferences": [""],
                    "assistant_nickname": ""
                }),
                &test_ctx(),
            )
            .await;
        assert!(result.is_err(), "all empty fields should result in error");
    }
}
