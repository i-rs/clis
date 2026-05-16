pub mod i_rs_cmd;
pub mod rig_tools;
pub mod search;

use serde_json::Value;

/// Get tool definitions for OpenAI-compatible chat completion APIs.
pub fn get_tool_schemas() -> Vec<Value> {
    rig_tools::tool_schemas()
}
