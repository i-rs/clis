use crate::memory::CrossSessionMemory;
use crate::provider::ToolCall;
use crate::tools::ToolRegistry;
use std::collections::HashMap;

const MAX_TOOL_RETRIES: u32 = 2;

pub(crate) struct ToolExecResult {
    pub(crate) tool_messages: Vec<(String, String, String)>,
}

pub(crate) async fn execute_tools(
    pending_tool_calls: &[ToolCall],
    tools: &ToolRegistry,
    retry_counts: &mut HashMap<String, u32>,
    tool_timeout_secs: u64,
    memory: &mut Option<CrossSessionMemory>,
) -> ToolExecResult {
    let mut tool_messages = Vec::new();

    for tc in pending_tool_calls {
        let mut result_str = String::new();
        let current_retries = *retry_counts.get(&tc.id).unwrap_or(&0);

        for attempt in 0..=current_retries {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            let tool = tools.get(&tc.name);
            let tc_clone = ToolCall { id: tc.id.clone(), name: tc.name.clone(), args: tc.args.clone() };

            let handle = tokio::spawn(async move {
                if let Some(tool) = tool {
                    match tc_clone.args.as_object() {
                        Some(obj) => tool.call(obj).await,
                        None => tool.call(&serde_json::Map::new()).await,
                    }
                } else {
                    Err(anyhow::anyhow!("Unknown tool: {}", tc_clone.name))
                }
            });

            result_str = match tokio::time::timeout(std::time::Duration::from_secs(tool_timeout_secs), handle).await {
                Ok(Ok(inner)) => match inner {
                    Ok(s) => crate::utils::truncate_output(&s, crate::error::MAX_TOOL_OUTPUT_BYTES),
                    Err(e) => format!("Error: {}", e),
                },
                Ok(Err(join_err)) => format!("Error: tool task panicked: {}", join_err),
                Err(_) => format!("Error: tool execution timed out ({}s)", tool_timeout_secs),
            };

            if !result_str.starts_with("Error:") {
                retry_counts.remove(&tc.id);
                break;
            }
        }

        if result_str.starts_with("Error:") {
            retry_counts.insert(tc.id.clone(), current_retries + 1);
        }

        if let Some(mem) = memory {
            mem.record_tool_use(&tc.name);
        }
        tool_messages.push((tc.name.clone(), tc.id.clone(), result_str));
    }
    ToolExecResult { tool_messages }
}

pub(crate) fn build_over_limit_message(retry_counts: &HashMap<String, u32>) -> Option<String> {
    let over_limit: Vec<_> = retry_counts.iter()
        .filter(|(_, c)| **c > MAX_TOOL_RETRIES)
        .map(|(id, c)| (id.clone(), *c))
        .collect();
    if over_limit.is_empty() { return None; }
    let details: Vec<String> = over_limit.iter()
        .map(|(id, c)| format!("tool_call_id='{}' ({}times)", id, c))
        .collect();
    Some(format!(
        "Tool failed {} consecutive times. Please reconsider:\n\
         1. Are the arguments correct?\n\
         2. Is there a different approach?\n\
         3. Is this tool needed at all?\n\
         Failed calls: {}",
        MAX_TOOL_RETRIES, details.join("; ")
    ))
}
