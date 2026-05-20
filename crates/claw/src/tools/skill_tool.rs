use serde_json::Value;
use crate::error::ClawError;
use crate::skill_store::SkillDefinition;
use crate::tools::{ClawTool, ToolContext};

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

impl ClawTool for SkillTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn parameter_schema(&self, _enabled_cli: &[&str]) -> Value {
        self.parameters
            .clone()
            .unwrap_or_else(default_schema)
    }

    fn execute(&self, _args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        if self.content.is_empty() {
            Ok("技能已激活，但未包含具体指令内容。".to_string())
        } else {
            Ok(self.content.clone())
        }
    }
}
