use crate::config::Config;
use crate::llm::{LlmEvent, StreamResult, ToolCallAcc};
use crate::provider::LlmProvider;
use crate::utils;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::mpsc;

/// Global MCP registry (initialized at startup from config).
pub(crate) static MCP_REGISTRY: OnceLock<crate::mcp::McpRegistry> = OnceLock::new();

/// Initialize the global MCP registry.
pub fn init_mcp(servers: &[crate::mcp::McpServerConfig]) {
    let registry = crate::mcp::McpRegistry::new(servers);
    let _ = MCP_REGISTRY.set(registry);
}

/// Prefix used to identify reminder system messages in the message list.
const REMINDER_PREFIX: &str = "注意：用户有以下即将到期或已到期的提醒事项";

/// Load system prompt from external file and inject dynamic layers.
///
/// Layers:
/// 1. Static behavior prompt (system.md)
/// 2. Tool index (from TOOL_INDEX static data)
/// 3. Hot tool docs (skill teach outputs for frequently used tools)
/// 4. User memory (cross-session preferences and history)
/// 5. User profile (name, preferences for onboarding)
pub(crate) fn build_system_prompt(
    tool_index: &str,
    hot_tools: &str,
    skills: &str,
    user_memory: &str,
    user_profile: &str,
) -> String {
    let mut prompt = include_str!("../../prompts/system.md").to_string();
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let weekday = now.format("%A").to_string();
    prompt = prompt
        .replace("{current_date}", &today)
        .replace("{current_weekday}", &weekday);

    prompt = prompt.replace("{{TOOL_INDEX}}", tool_index);
    prompt = prompt.replace("{{HOT_TOOLS}}", hot_tools);
    prompt = prompt.replace("{{SKILLS}}", skills);
    prompt = prompt.replace("{{USER_MEMORY}}", user_memory);
    prompt = prompt.replace("{{USER_PROFILE}}", user_profile);

    // Collapse 3+ consecutive newlines into 2 (one blank line)
    // Ensures empty dynamic content doesn't leave gaps in the stable prefix
    while prompt.contains("\n\n\n") {
        prompt = prompt.replace("\n\n\n", "\n\n");
    }

    prompt
}

/// Convert app messages to API-compatible message list.
/// If `saved_api_messages` exists, reuse them as base (preserving tool call context)
/// and only append the new user message.
/// If `system_prompt_override` is provided, it replaces the default system prompt.
pub fn build_messages(
    app_messages: &[crate::app::Message],
    user_text: &str,
    saved_api_messages: &Option<Vec<Value>>,
    tool_frequency: &HashMap<String, usize>,
    tool_index: &str,
    hot_tools: &str,
    skills: &str,
    user_memory: &str,
    user_profile: &str,
    reminder_text: Option<&str>,
    system_prompt_override: Option<&str>,
) -> Vec<Value> {
    // Helper: remove stale reminder system message at index 1 if present
    let remove_reminder_msg = |msgs: &mut Vec<Value>| {
        if msgs.len() > 1
            && msgs[1].get("role").and_then(|r| r.as_str()) == Some("system")
            && msgs[1]
                .get("content")
                .and_then(|c| c.as_str())
                .map_or(false, |c| c.starts_with(REMINDER_PREFIX))
        {
            msgs.remove(1);
        }
    };

    // Helper: inject reminder system message at index 1
    let inject_reminder = |msgs: &mut Vec<Value>, text: &str| {
        if !text.is_empty() {
            msgs.insert(
                1,
                serde_json::json!({
                    "role": "system",
                    "content": format!("{}：\n{}", REMINDER_PREFIX, text),
                }),
            );
        }
    };

    if let Some(prev_msgs) = saved_api_messages {
        // Reuse saved API messages (has full context including tool calls)
        let mut msgs = prev_msgs.clone();
        // Remove stale reminder message before injecting fresh one
        remove_reminder_msg(&mut msgs);
        // Remove trailing user message if exists (from previous turn)
        if msgs.len() > 1
            && msgs
                .last()
                .and_then(|m| m.get("role").and_then(|r| r.as_str()))
                == Some("user")
        {
            msgs.pop();
        }
        msgs.push(serde_json::json!({"role": "user", "content": user_text}));

        // Inject fresh reminder
        if let Some(rt) = reminder_text {
            inject_reminder(&mut msgs, rt);
        }

        // Smart compress: preserve skill teach docs + recent conversation context
        smart_compress(&mut msgs, tool_frequency, 5, 15);
        return msgs;
    }

    // First turn: build from scratch
    let system_prompt = system_prompt_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| build_system_prompt(tool_index, hot_tools, skills, user_memory, user_profile));

    let mut msgs = vec![serde_json::json!({
        "role": "system",
        "content": system_prompt,
    })];

    // Inject reminder right after system prompt
    if let Some(rt) = reminder_text {
        inject_reminder(&mut msgs, rt);
    }

    // Keep last ~8 display messages for context
    let max_turns = 8;
    let start = app_messages.len().saturating_sub(max_turns);

    for msg in &app_messages[start..] {
        match msg {
            crate::app::Message::User { text } => {
                msgs.push(serde_json::json!({"role": "user", "content": text}));
            }
            crate::app::Message::Assistant { text } if !text.is_empty() => {
                msgs.push(serde_json::json!({"role": "assistant", "content": text}));
            }
            _ => {}
        }
    }

    msgs.push(serde_json::json!({"role": "user", "content": user_text}));
    msgs
}

/// Execute a parsed tool call and return the result.
/// Tries built-in tools first, then falls back to MCP-discovered tools.
pub(crate) fn execute_tool_call(
    name: &str,
    args: &Value,
) -> String {
    // Try built-in tools first
    let registry = crate::tools::ToolRegistry::new();
    if registry.tool_exists(name) {
        // Tool found — let it execute; propagate real CLI error (not "unknown tool")
        return registry.execute(name, args).unwrap_or_else(|e| e);
    }

    // Try MCP-discovered tools
    if let Some(mcp) = MCP_REGISTRY.get() {
        for (client_idx, tool_def) in &mcp.tools {
            if tool_def.name == name {
                if let Some(client) = mcp.clients.get(*client_idx) {
                    return client.call_tool(name, args)
                        .unwrap_or_else(|e| format!("MCP 错误: {}", e));
                }
            }
        }
    }

    format!("错误: 未知工具 {}", name)
}

/// A detected skill teach doc pair in the API message list.
struct TeachPair {
    assist_idx: usize,
    result_idx: usize,
    tool_name: String,
}

/// Find skill teach doc pairs (i_rs → skill command) by scanning backwards.
/// Deduplicates by tool name, keeping the latest occurrence of each tool.
fn find_teach_pairs(msgs: &[Value]) -> Vec<TeachPair> {
    use std::collections::HashSet;
    let mut pairs: Vec<TeachPair> = Vec::new();
    let mut seen_tools: HashSet<String> = HashSet::new();

    let mut i = msgs.len();
    while i > 0 {
        i -= 1;
        if let Some(tool_calls) = msgs[i].get("tool_calls").and_then(|t| t.as_array()) {
            for tc in tool_calls {
                if let Some(name) = tc.get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                {
                    if name != "i_rs" { continue; }
                    let args_str = tc.get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|a| a.as_str())
                        .unwrap_or("");
                    if let Ok(parsed) = serde_json::from_str::<Value>(args_str) {
                        let cmd = parsed.get("command").and_then(|c| c.as_str()).unwrap_or("");
                        if cmd != "skill" { continue; }
                        let tool_name = parsed.get("tool").and_then(|t| t.as_str()).unwrap_or("unknown").to_string();

                        if seen_tools.contains(&tool_name) { continue; }
                        seen_tools.insert(tool_name.clone());

                        if i + 1 < msgs.len() && msgs[i + 1].get("role").and_then(|r| r.as_str()) == Some("tool") {
                            pairs.push(TeachPair { assist_idx: i, result_idx: i + 1, tool_name });
                        }
                    }
                }
            }
        }
    }
    pairs
}

/// Score teach pairs by global cross-session frequency + position recency.
/// Returns indices into the pairs list sorted by score (highest first).
///
/// Frequency is weighted 10x so that tools used across multiple sessions
/// are strongly preferred over one-off tool learns.
fn score_teach_pairs(pairs: &[TeachPair], tool_frequency: &HashMap<String, usize>) -> Vec<usize> {
    let max_recency = pairs.len().max(1);
    let mut scored: Vec<(usize, usize)> = pairs.iter().enumerate().map(|(pos, pair)| {
        let freq = tool_frequency.get(&pair.tool_name).copied().unwrap_or(0);
        let recency = max_recency - pos;
        (freq * 10 + recency, pos)
    }).collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, pos)| pos).collect()
}

/// Smart compress API message list, preserving high-value content.
///
/// Strategy:
/// 1. Always keep system message
/// 2. Find skill teach doc pairs, score by cross-session frequency + recency
/// 3. Keep top-scoring pairs that fall outside the recent window
/// 4. Keep recent conversation messages intact
/// 5. Drop old dialog that lacks teach value
///
/// This implements "predict which tools are worth keeping" by using
/// cross-session usage frequency as the primary signal (weighted 10x)
/// and recency as the secondary signal.
fn smart_compress(
    msgs: &mut Vec<Value>,
    tool_frequency: &HashMap<String, usize>,
    max_teach_docs: usize,
    recent_keep: usize,
) {
    if msgs.len() <= 1 + recent_keep {
        return;
    }

    use std::collections::HashSet;

    let teach_pairs = find_teach_pairs(msgs);
    let sorted_ranks = score_teach_pairs(&teach_pairs, tool_frequency);

    let mut preserve: HashSet<usize> = HashSet::new();
    preserve.insert(0); // system message

    // Keep top-scoring teach pairs
    for rank in 0..max_teach_docs.min(sorted_ranks.len()) {
        let pos = sorted_ranks[rank];
        preserve.insert(teach_pairs[pos].assist_idx);
        preserve.insert(teach_pairs[pos].result_idx);
    }

    // Keep recent conversation messages
    let recent_start = msgs.len().saturating_sub(recent_keep);
    for idx in recent_start..msgs.len() {
        preserve.insert(idx);
    }

    // Ensure tool_call + tool result pairs are kept together to prevent
    // orphaned tool messages ("role='tool' must follow tool_calls" error).
    // Scan both directions: if a "tool" result is kept but its preceding
    // tool_call was dropped, restore the tool_call; and vice versa.
    let mut changed = true;
    while changed {
        changed = false;
        for idx in 0..msgs.len() {
            let kept = preserve.contains(&idx);
            if !kept {
                continue;
            }
            // If this is a tool result, ensure preceding tool_call is kept
            if msgs[idx].get("role").and_then(|r| r.as_str()) == Some("tool")
                && idx > 0
                && msgs[idx - 1].get("tool_calls").is_some()
                && !preserve.contains(&(idx - 1))
            {
                preserve.insert(idx - 1);
                changed = true;
            }
            // If this is a tool_call, ensure following tool result is kept
            if msgs[idx].get("tool_calls").is_some()
                && idx + 1 < msgs.len()
                && msgs[idx + 1].get("role").and_then(|r| r.as_str()) == Some("tool")
                && !preserve.contains(&(idx + 1))
            {
                preserve.insert(idx + 1);
                changed = true;
            }
        }
    }

    // Build compressed message list
    let mut new_msgs: Vec<Value> = Vec::with_capacity(preserve.len());
    for idx in 0..recent_start {
        if preserve.contains(&idx) {
            new_msgs.push(msgs[idx].clone());
        }
    }
    for idx in recent_start..msgs.len() {
        new_msgs.push(msgs[idx].clone());
    }

    *msgs = new_msgs;
}

/// Compress API messages after a conversation turn completes.
///
/// Preserves top 5 skill teach docs (scored by cross-session frequency + recency)
/// and the last 20 conversation messages for context.
pub fn compress_api_messages(
    msgs: &mut Vec<Value>,
    tool_frequency: &HashMap<String, usize>,
) {
    smart_compress(msgs, tool_frequency, 5, 20);
}

/// Main chat loop: stream, handle tool calls, continue until done
pub async fn chat_loop(
    provider: Box<dyn LlmProvider>,
    config: Config,
    messages: Vec<Value>,
    tx: mpsc::UnboundedSender<LlmEvent>,
) {
    let enabled = if config.enabled_tools.is_empty() {
        None
    } else {
        Some(&config.enabled_tools)
    };
    let mut tool_schemas = crate::tools::ToolRegistry::new().enabled_schemas(enabled);
    // Append MCP tool schemas if available
    if let Some(mcp) = MCP_REGISTRY.get() {
        for (client_idx, tool_def) in &mcp.tools {
            if let Some(_client) = mcp.clients.get(*client_idx) {
                let schema = crate::tools::mcp_tools::mcp_schema_to_openai(tool_def);
                tool_schemas.push(schema);
            }
        }
    }
    let mut msgs = messages;
    let mut retry_counts: HashMap<String, u32> = HashMap::new();
    const MAX_RETRIES: u32 = 2;
    const MAX_ROUNDS: u32 = 20;
    let mut round_count = 0u32;

    loop {
        round_count += 1;
        if round_count > MAX_ROUNDS {
            let _ = tx.send(LlmEvent::Error("已达最大执行轮数限制 (20)，已停止循环。".to_string()));
            break;
        }
        let _ = tx.send(LlmEvent::NewRound);
        let _ = tx.send(LlmEvent::Status("🤔 思考中…".to_string()));

        match provider.stream_chat(&msgs, &tool_schemas, &tx).await {
            Ok(StreamResult::Text(usage, text)) => {
                if !text.is_empty() {
                    msgs.push(serde_json::json!({
                        "role": "assistant",
                        "content": text,
                    }));
                }
                let _ = tx.send(LlmEvent::Done(msgs.clone(), usage));
                break;
            }
            Ok(StreamResult::ToolCalls(calls, reasoning_content)) => {
                let tool_calls_array: Vec<Value> = calls
                    .iter()
                    .map(|(tc, _)| {
                        serde_json::json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments,
                            }
                        })
                    })
                    .collect();

                let mut assistant_msg = serde_json::json!({
                    "role": "assistant",
                    "content": null,
                    "tool_calls": tool_calls_array,
                });
                // DeepSeek requires reasoning_content to be echoed back
                if !reasoning_content.is_empty() {
                    assistant_msg["reasoning_content"] = Value::String(reasoning_content);
                }
                msgs.push(assistant_msg);

                // Parallel execute all tool calls
                let total = calls.len();
                let _ = tx.send(LlmEvent::Status(format!("⚡ 并行执行 {} 个工具...", total)));

                let mut handles = Vec::new();
                for (step, (tc, args)) in calls.into_iter().enumerate() {
                    let tx = tx.clone();
                    let tc_name = tc.name.clone();
                    let args_str = serde_json::to_string(&args).unwrap_or_default();
                    let args_for_blocking = args.clone();
                    handles.push(tokio::spawn(async move {
                        let result = tokio::task::spawn_blocking(move || {
                            execute_tool_call(&tc_name, &args_for_blocking)
                        })
                        .await
                        .unwrap_or_else(|e| format!("错误: 内部错误: {}", e));

                        let _ = tx.send(LlmEvent::ToolExecuted {
                            name: tc.name.clone(),
                            args: args_str,
                            result: utils::smart_truncate(&result, 200),
                            step,
                            total_steps: total,
                        });

                        (tc, args, result)
                    }));
                }

                // Collect all results in order
                let mut all_results: Vec<(ToolCallAcc, Value, String)> = Vec::new();
                for handle in handles {
                    if let Ok(r) = handle.await {
                        all_results.push(r);
                    }
                }

                // Check for errors and track retry counts
                let mut should_retry = false;
                for (tc, _, result) in &all_results {
                    if result.starts_with("错误:") {
                        let count = retry_counts.entry(tc.id.clone()).or_insert(0);
                        *count += 1;
                        if *count <= MAX_RETRIES {
                            should_retry = true;
                        }
                    }
                }

                if should_retry {
                    // Push all results so LLM sees what succeeded/failed
                    for (tc, _args, result) in &all_results {
                        msgs.push(serde_json::json!({ "role": "tool", "tool_call_id": tc.id, "content": utils::smart_truncate(result, 500) }));
                    }
                    // Add retry guidance
                    msgs.push(serde_json::json!({
                        "role": "system",
                        "content": "部分工具调用返回错误，请修正参数后重试。".to_string(),
                    }));
                } else {
                    // Add reflection for max-retries-exceeded errors, then push all results
                    for (tc, _args, result) in &all_results {
                        if result.starts_with("错误:") {
                            msgs.push(serde_json::json!({
                                "role": "system",
                                "content": format!(
                                    "工具 '{}' 连续 {} 次调用失败。请反思：\n\
                                     1. 参数是否正确？\n\
                                     2. 是否需要换一种方式完成用户请求？\n\
                                     3. 是否不需要这个工具，用其他方式回答用户？\n\
                                     错误信息：{}",
                                    tc.name, MAX_RETRIES, result
                                ),
                            }));
                        }
                        let trimmed = utils::smart_truncate(result, 500);
                        msgs.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": tc.id,
                            "content": trimmed,
                        }));
                    }
                }
                // Continue loop: send tool results back to LLM
            }
            Err(e) => {
                let _ = tx.send(LlmEvent::Error(format!("{}", e)));
                break;
            }
        }
    }
}
