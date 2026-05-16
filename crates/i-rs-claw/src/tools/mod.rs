pub mod i_rs_cmd;
pub mod search;

use serde_json::Value;

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

pub fn get_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "i_rs",
            description: concat!(
                "执行 i-rs CLI 命令来管理用户的个人数据。\n",
                "命令格式: i-rs <tool> <command> [args...]\n",
                "\n",
                "== 系统约束：先学习，再执行 ==\n",
                "系统强制要求：执行数据操作前必须先学习该工具。\n",
                "未学习的工具直接执行会被系统自动拒绝。\n",
                "必须先调用 skill teach 或 example 来学习。\n",
                "\n",
                "== 学习方式 ==\n",
                "tool=\"xxx\", command=\"skill\", args=[\"teach\"] 输出完整教学文档\n",
                "tool=\"xxx\", command=\"example\", args=[] 输出使用示例\n",
                "\n",
                "示例学习流程：\n",
                "1. 学习: tool=weight, command=skill, args=[\"teach\"]\n",
                "2. 执行: tool=weight, command=add, args=[\"2025-01-15\", \"75\"]\n",
                "\n",
                "注意: args 每个参数独立元素，不要合并值。DATE 必须用 YYYY-MM-DD 格式"
            ),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "tool": {
                        "type": "string",
                        "description": "i-rs 工具名称，即 i-rs 命令后的第一个参数。如 weight(体重)、run(跑步)、sleep(睡眠)、ledger(记账)、todo(待办)、mood(心情)、habit(习惯)、water(饮水)等。不是 'record' 或 'data'"
                    },
                    "command": {
                        "type": "string",
                        "description": "要执行的子命令。add=添加, list=列出, delete=删除, stats=统计, chart/calendar=图表, done=完成, checkin=打卡, skill=学习工具用法(带 args=[\"teach\"] 输出教学文档)。按工具支持的功能选择"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "命令参数（字符串数组）。按 CLI 工具需要的顺序传入。args=[\"2025-01-15\", \"75\"] 或 args=[\"teach\"]。每个参数独立元素。日期必须用 YYYY-MM-DD 格式"
                    },
                    "explanation": {
                        "type": "string",
                        "description": "用中文简短解释当前操作，方便用户理解"
                    }
                },
                "required": ["tool", "command", "explanation"]
            }),
        },
        ToolDef {
            name: "search_tools",
            description: "搜索哪些 i-rs 工具可以处理特定的需求。当你不太确定该用哪个工具时使用",
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索关键词，描述你想做什么"
                    }
                },
                "required": ["query"]
            }),
        },
    ]
}

pub fn get_tool_schemas(tools: &[ToolDef]) -> Vec<Value> {
    tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        })
        .collect()
}
