use crate::mcp::{McpClient, McpToolDefinition};
use crate::tools::ClawTool;
use serde_json::Value;

/// Convert an MCP tool definition to an OpenAI-compatible tool schema.
pub fn mcp_schema_to_openai(tool_def: &McpToolDefinition) -> Value {
    let mut parameters = tool_def.input_schema.clone();
    if parameters.get("additionalProperties").is_none() {
        parameters["additionalProperties"] = Value::Bool(false);
    }
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool_def.name,
            "description": tool_def.description,
            "parameters": parameters,
        }
    })
}

/// Wraps an MCP-discovered tool as a built-in `ClawTool`.
///
/// Each `McpToolWrapper` corresponds to one tool from one MCP server.
/// The `client` is cloned (cheap, Arc-based) from the shared MCP client.
#[allow(dead_code)]
pub struct McpToolWrapper {
    definition: McpToolDefinition,
    client: McpClient,
}

impl McpToolWrapper {
    pub fn new(definition: McpToolDefinition, client: McpClient) -> Self {
        Self { definition, client }
    }
}

impl ClawTool for McpToolWrapper {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn description(&self) -> &str {
        if self.definition.description.is_empty() {
            &self.definition.name
        } else {
            &self.definition.description
        }
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        // MCP JSON Schema → OpenAI function parameters format.
        // Most MCP tools use a JSON Schema that's already compatible:
        // {"type": "object", "properties": {...}, "required": [...]}
        //
        // We add "additionalProperties": false for strict mode compatibility.
        let schema = &self.definition.input_schema;
        let mut adapted = schema.clone();
        if adapted.get("additionalProperties").is_none() {
            adapted["additionalProperties"] = Value::Bool(false);
        }
        adapted
    }

    fn execute(&self, args: &Value) -> Result<String, String> {
        let result = self.client.call_tool(&self.definition.name, args)?;
        Ok(result)
    }
}
