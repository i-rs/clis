use crate::agent::tool_cache::ToolResultCache;
use crate::memory::CrossSessionMemory;
use crate::provider::ToolCall;
use crate::tools::ToolRegistry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    cache: Option<&Arc<Mutex<ToolResultCache>>>,
) -> ToolExecResult {
    let mut tool_messages = Vec::with_capacity(pending_tool_calls.len());

    let mut handles = Vec::with_capacity(pending_tool_calls.len());
    for tc in pending_tool_calls {
        let args_json = serde_json::to_string(&tc.args).unwrap_or_default();

        // Check cache for read-only tools
        if ToolResultCache::is_cacheable(&tc.name)
            && let Some(cache) = cache
            && let Some(cached) = cache
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get(&tc.name, &args_json)
        {
            tool_messages.push((tc.name.clone(), tc.id.clone(), cached.to_string()));
            continue;
        }

        // Invalidate cache for mutator tools
        if ToolResultCache::is_mutator(&tc.name)
            && let Some(cache) = cache
        {
            cache
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .invalidate_all();
        }

        let tool = tools.get(&tc.name);
        let tc_clone = ToolCall {
            id: tc.id.clone(),
            name: tc.name.clone(),
            args: tc.args.clone(),
        };
        let cache_clone = cache.map(Arc::clone);

        let handle = tokio::spawn(async move {
            let result = if let Some(tool) = tool {
                match tc_clone.args.as_object() {
                    Some(obj) => tool.call(obj).await,
                    None => tool.call(&serde_json::Map::new()).await,
                }
            } else {
                Err(anyhow::anyhow!("Unknown tool: {}", tc_clone.name))
            };

            // Cache successful results for read-only tools
            if let Ok(ref val) = result
                && ToolResultCache::is_cacheable(&tc_clone.name)
                && let Some(cache) = cache_clone
            {
                let args_json = serde_json::to_string(&tc_clone.args).unwrap_or_default();
                cache.lock().unwrap_or_else(|e| e.into_inner()).insert(
                    &tc_clone.name,
                    &args_json,
                    val.clone(),
                );
            }

            result
        });
        handles.push((tc.id.clone(), tc.name.clone(), tc.args.clone(), handle));
    }

    let timeout = std::time::Duration::from_secs(tool_timeout_secs);
    for (id, name, args, handle) in handles {
        let mut result_str = run_tool_handle(handle, timeout, &id, tool_timeout_secs).await;

        let mut retries = 0;
        while result_str.starts_with("Error:") && retries < MAX_TOOL_RETRIES {
            retries += 1;
            let tool = tools.get(&name);
            let retry_args = args.clone();
            let retry_name = name.clone();
            let cache_clone = cache.map(Arc::clone);
            let retry_handle = tokio::spawn(async move {
                let result = match tool {
                    Some(t) => match retry_args.as_object() {
                        Some(obj) => t.call(obj).await,
                        None => t.call(&serde_json::Map::new()).await,
                    },
                    None => Err(anyhow::anyhow!("Unknown tool: {}", retry_name)),
                };
                if let Ok(ref val) = result
                    && ToolResultCache::is_cacheable(&retry_name)
                    && let Some(cache) = cache_clone
                {
                    let args_json = serde_json::to_string(&retry_args).unwrap_or_default();
                    cache.lock().unwrap_or_else(|e| e.into_inner()).insert(
                        &retry_name,
                        &args_json,
                        val.clone(),
                    );
                }
                result
            });
            result_str = run_tool_handle(retry_handle, timeout, &id, tool_timeout_secs).await;
        }

        if result_str.starts_with("Error:") {
            let current_retries = *retry_counts.get(&id).unwrap_or(&0);
            retry_counts.insert(id.clone(), current_retries + 1);
        } else {
            retry_counts.remove(&id);
        }

        if let Some(mem) = memory {
            mem.record_tool_use(&name);
        }
        tool_messages.push((name.clone(), id.clone(), result_str));
    }

    ToolExecResult { tool_messages }
}

async fn run_tool_handle(
    handle: tokio::task::JoinHandle<anyhow::Result<String>>,
    timeout: std::time::Duration,
    _id: &str,
    tool_timeout_secs: u64,
) -> String {
    match tokio::time::timeout(timeout, handle).await {
        Ok(Ok(inner)) => match inner {
            Ok(s) => crate::utils::truncate_output(&s, crate::error::MAX_TOOL_OUTPUT_BYTES),
            Err(e) => format!("Error: {}", e),
        },
        Ok(Err(join_err)) => format!("Error: tool task panicked: {}", join_err),
        Err(_) => format!("Error: tool execution timed out ({}s)", tool_timeout_secs),
    }
}

pub(crate) fn build_over_limit_message(retry_counts: &HashMap<String, u32>) -> Option<String> {
    let over_limit: Vec<_> = retry_counts
        .iter()
        .filter(|(_, c)| **c > MAX_TOOL_RETRIES)
        .map(|(id, c)| (id.clone(), *c))
        .collect();
    if over_limit.is_empty() {
        return None;
    }
    let details: Vec<String> = over_limit
        .iter()
        .map(|(id, c)| format!("tool_call_id='{}' ({}times)", id, c))
        .collect();
    Some(format!(
        "Tool failed {} consecutive times. Please reconsider:\n\
         1. Are the arguments correct?\n\
         2. Is there a different approach?\n\
         3. Is this tool needed at all?\n\
         Failed calls: {}",
        MAX_TOOL_RETRIES,
        details.join("; ")
    ))
}
