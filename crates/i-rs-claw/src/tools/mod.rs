pub mod i_rs_cmd;
pub mod rig_tools;
pub mod search;

use serde_json::Value;
use std::collections::HashSet;

/// Get tool definitions for OpenAI-compatible chat completion APIs.
/// Filters by `enabled_tools` if provided (empty set = all).
pub fn get_tool_schemas(enabled: Option<&HashSet<String>>) -> Vec<Value> {
    rig_tools::tool_schemas(enabled)
}
