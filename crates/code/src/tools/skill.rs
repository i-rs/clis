use crate::skill_store::SkillStore;
use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Map, Value, json};

pub struct SkillTool;

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill"
    }
    fn description(&self) -> &str {
        "List or load skills. Skills provide specialized instructions for specific tasks. Use 'list' to discover available skills, then 'get <name>' to load the full instructions."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "skill",
                "description": "List available skills or load a specific skill's instructions.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["list", "get"],
                            "description": "'list' to show all available skills, 'get' to load a specific skill's content"
                        },
                        "name": {
                            "type": "string",
                            "description": "Skill name to load (required for action='get')"
                        }
                    },
                    "required": ["action"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        match action {
            "list" => {
                let store = SkillStore::new();
                let skills = store.list();
                if skills.is_empty() {
                    return Ok(
                        "No skills installed. Create one with `i-rs-code skill create <name>`."
                            .to_string(),
                    );
                }
                let mut out = format!("Available skills ({}):\n", skills.len());
                for skill in &skills {
                    out.push_str(&format!(
                        "  {}  {}  — {}\n",
                        "📋", skill.name, skill.description
                    ));
                }
                out.push_str(
                    "\nUse `skill` tool with action='get' and name='<name>' to load a skill.",
                );
                Ok(out)
            }
            "get" => {
                let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    crate::error::ToolError::invalid_args(
                        "Missing 'name' parameter for action='get'",
                    )
                })?;
                let store = SkillStore::new();
                match store.get(name) {
                    Some(skill) => Ok(format!("# Skill: {}\n\n{}", skill.name, skill.content)),
                    None => Ok(format!(
                        "Skill '{}' not found. Use `skill` tool with action='list' to see available skills.",
                        name
                    )),
                }
            }
            _ => Ok(format!("Unknown action: {}. Use 'list' or 'get'.", action)),
        }
    }
}
