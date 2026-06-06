use crate::error::ClawError;
use crate::skill_store::SkillDefinition;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

/// Wraps a user-defined skill as a callable tool.
///
/// Skills with `parameters` defined in TOML frontmatter are registered
/// as `skill_{name}` tools. When called by the LLM, the tool returns
/// the skill's content (without the frontmatter block).
///
/// This enables on-demand skill loading: instead of injecting all
/// skills into the system prompt, the LLM can selectively invoke
/// skills as needed, saving tokens and keeping prompts focused.
pub struct SkillTool {
    tool_name: String,
    tool_description: String,
    parameters: Option<Value>,
    content: String,
}

impl SkillTool {
    pub fn new(definition: SkillDefinition) -> Self {
        let desc = if definition.description.is_empty() {
            definition.name.clone()
        } else {
            definition.description.clone()
        };
        Self {
            tool_name: format!("skill_{}", definition.name),
            tool_description: desc,
            parameters: definition.parameters,
            content: definition.content,
        }
    }
}

/// Default empty JSON Schema for skills without explicit parameters.
fn default_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {},
        "additionalProperties": false,
    })
}

#[async_trait::async_trait]
impl ClawTool for SkillTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn parameter_schema(&self, _enabled_cli: &[&str]) -> Value {
        self.parameters.clone().unwrap_or_else(default_schema)
    }

    async fn execute(&self, _args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        if self.content.is_empty() {
            Ok("技能已激活，但未包含具体指令内容。".to_string())
        } else {
            Ok(self.content.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_store::SkillDefinition;
    use serde_json::json;

    #[test]
    fn test_skill_tool_name_prefix() {
        let def = SkillDefinition {
            name: "my_skill".to_string(),
            description: String::new(),
            parameters: None,
            content: String::new(),
        };
        let tool = SkillTool::new(def);
        assert_eq!(tool.name(), "skill_my_skill");
    }

    #[test]
    fn test_skill_tool_description_fallback() {
        let def = SkillDefinition {
            name: "my_skill".to_string(),
            description: String::new(),
            parameters: None,
            content: String::new(),
        };
        let tool = SkillTool::new(def);
        assert_eq!(tool.description(), "my_skill");
    }

    #[test]
    fn test_skill_tool_description_custom() {
        let def = SkillDefinition {
            name: "my_skill".to_string(),
            description: "Custom desc".to_string(),
            parameters: None,
            content: String::new(),
        };
        let tool = SkillTool::new(def);
        assert_eq!(tool.description(), "Custom desc");
    }

    #[test]
    fn test_skill_tool_parameter_schema_default() {
        let def = SkillDefinition {
            name: "s".to_string(),
            description: String::new(),
            parameters: None,
            content: String::new(),
        };
        let tool = SkillTool::new(def);
        let schema = tool.parameter_schema(&[]);
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
    }

    #[test]
    fn test_skill_tool_parameter_schema_custom() {
        let def = SkillDefinition {
            name: "s".to_string(),
            description: String::new(),
            parameters: Some(json!({"type": "object", "properties": {"x": {"type": "string"}}})),
            content: String::new(),
        };
        let tool = SkillTool::new(def);
        let schema = tool.parameter_schema(&[]);
        assert_eq!(schema["properties"]["x"]["type"], "string");
    }

    #[tokio::test]
    async fn test_skill_tool_execute_with_content() {
        let def = SkillDefinition {
            name: "s".to_string(),
            description: String::new(),
            parameters: None,
            content: "Execute this instruction".to_string(),
        };
        let tool = SkillTool::new(def);
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: crate::providers::shared_client(),
            delegate_runtime: None,
        };
        let result = tool.execute(&json!({}), &ctx).await.unwrap();
        assert_eq!(result, "Execute this instruction");
    }

    #[tokio::test]
    async fn test_skill_tool_execute_empty_content() {
        let def = SkillDefinition {
            name: "s".to_string(),
            description: String::new(),
            parameters: None,
            content: String::new(),
        };
        let tool = SkillTool::new(def);
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: crate::providers::shared_client(),
            delegate_runtime: None,
        };
        let result = tool.execute(&json!({}), &ctx).await.unwrap();
        assert!(result.contains("技能已激活"));
    }
}
