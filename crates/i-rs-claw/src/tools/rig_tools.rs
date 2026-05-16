use serde_json::Value;
use std::collections::HashSet;

use crate::tools::search::{is_tool_enabled, TOOL_INDEX};

/// Get the JSON schema definitions for all tools.
///
/// These are manually defined to avoid Rig's #[tool] derive complexity.
/// The schemas are OpenAI-compatible function calling format.
/// If `enabled` is Some, only include tools in that set.
pub fn tool_schemas(enabled: Option<&HashSet<String>>) -> Vec<Value> {
    let enabled_tools: Vec<&str> = TOOL_INDEX.iter()
        .map(|(name, _)| *name)
        .filter(|t| is_tool_enabled(t, enabled))
        .collect();

    vec![
        // i_rs tool — the enum variants filtered by enabled_tools
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "i_rs",
                "description": "执行 i-rs CLI 命令来管理用户的个人数据",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "tool": {
                            "type": "string",
                            "enum": enabled_tools,
                            "description": "i-rs 工具名称（i-rs 后的第一个参数）"
                        },
                        "command": {
                            "type": "string",
                            "description": "子命令。常见的有 add/list/delete/stats。还有 skill（用来学习工具用法）"
                        },
                        "args": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "参数数组，按工具需要的顺序传入。每个参数独立元素，不要合并值"
                        },
                        "explanation": {
                            "type": "string",
                            "description": "用中文解释当前操作"
                        }
                    },
                    "required": ["tool", "command"]
                }
            }
        }),
        // search_tools tool (always enabled)
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "search_tools",
                "description": "搜索哪些 i-rs 工具可以处理特定的需求",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "搜索关键词，描述用户想要完成的任务"
                        }
                    },
                    "required": ["query"]
                }
            }
        }),
    ]
}
