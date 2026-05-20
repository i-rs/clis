use crate::config::Config;
use crate::llm::{LlmEvent, StreamResult};
use crate::mcp::McpRegistry;
use crate::provider::LlmProvider;
use crate::skill_store::SkillDefinition;
use crate::utils;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Prefix used to identify reminder system messages in the message list.
const REMINDER_PREFIX: &str = "注意：用户有以下即将到期或已到期的提醒事项";

/// ReAct mode instruction (default) — no upfront planning, just act step by step.
const REACT_PROMPT: &str = "\
当用户请求涉及 **2 个或以上不同工具调用** 时：
1. 无需预先规划整个流程，直接开始执行第一步
2. 根据上一步的结果自然决定下一步
3. 按顺序依次执行，每次调用一个工具
4. 所有步骤完成后给出总结
";

/// Plan-then-Execute mode instruction (experimental).
const PLAN_THEN_EXECUTE_PROMPT: &str = "\
当用户请求涉及 **2 个或以上不同工具调用** 时，必须使用执行计划模式。

### Plan-then-Execute 标准流程

**步骤 1：分析需求**
- 识别用户需求的几个步骤
- 为每个步骤分配对应的工具

**步骤 2：输出结构化计划**
在回复开头输出计划（纯文本格式）：
📋 执行计划：
1. [步骤描述] → 工具名/命令
2. [步骤描述] → 工具名/命令
3. [步骤描述] → 工具名/命令

**步骤 3：逐步骤执行**
- 每执行完一步，更新计划状态：在回复中标注 ✅ 完成
- 如某步失败，输出 ❌ 失败原因，然后决定重试或跳过
- 所有步骤完成后输出总结

### 计划跟踪格式
在涉及多步操作时，每次回复前部显示当前进度：
📋 计划进度: 1/3 ✅ → 2/3 🔄 → 3/3 ✅
";

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
    plan_then_execute: bool,
) -> String {
    let mut prompt = include_str!("../../prompts/system.md").to_string();
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let weekday = now.format("%A").to_string();
    prompt = prompt
        .replace("{current_date}", &today)
        .replace("{current_weekday}", &weekday);

    let plan_mode = if plan_then_execute { PLAN_THEN_EXECUTE_PROMPT } else { REACT_PROMPT };
    prompt = prompt.replace("{{PLAN_MODE}}", plan_mode);

    prompt = prompt.replace("{{TOOL_INDEX}}", tool_index);
    prompt = prompt.replace("{{HOT_TOOLS}}", hot_tools);
    prompt = prompt.replace("{{SKILLS}}", skills);
    prompt = prompt.replace("{{USER_MEMORY}}", user_memory);
    prompt = prompt.replace("{{USER_PROFILE}}", user_profile);

    // Collapse 3+ consecutive newlines into 2 (one blank line)
    // Ensures empty dynamic content doesn't leave gaps in the stable prefix.
    // Use a simple approach: handle 4+ first, then exactly 3, to avoid O(n²) while-loop.
    prompt = prompt.replace("\n\n\n\n", "\n\n");
    prompt = prompt.replace("\n\n\n", "\n\n");

    prompt
}

/// Parameters for building API-compatible message lists.
///
/// Consolidates the many parameters of `build_messages` into a single struct
/// to improve readability and make the call site more maintainable.
pub struct MessageBuildParams<'a> {
    pub app_messages: &'a [crate::app::Message],
    pub user_text: &'a str,
    pub saved_api_messages: &'a Option<Vec<Value>>,
    pub tool_frequency: &'a HashMap<String, usize>,
    pub tool_index: &'a str,
    pub hot_tools: &'a str,
    pub skills: &'a str,
    pub user_memory: &'a str,
    pub user_profile: &'a str,
    pub reminder_text: Option<&'a str>,
    pub system_prompt_override: Option<&'a str>,
    pub plan_then_execute: bool,
    pub max_conversation_turns: usize,
}

/// Convert app messages to API-compatible message list.
/// If `saved_api_messages` exists, reuse them as base (preserving tool call context)
/// and only append the new user message.
/// If `system_prompt_override` is provided, it replaces the default system prompt.
pub fn build_messages(params: MessageBuildParams) -> Vec<Value> {
    // Helper: remove stale reminder system message at index 1 if present
    let remove_reminder_msg = |msgs: &mut Vec<Value>| {
        if msgs.len() > 1
            && msgs[1].get("role").and_then(|r| r.as_str()) == Some("system")
            && msgs[1]
                .get("content")
                .and_then(|c| c.as_str())
                .is_some_and(|c| c.starts_with(REMINDER_PREFIX))
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

    if let Some(prev_msgs) = params.saved_api_messages {
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
        msgs.push(serde_json::json!({"role": "user", "content": params.user_text}));

        // Inject fresh reminder
        if let Some(rt) = params.reminder_text {
            inject_reminder(&mut msgs, rt);
        }

        // Smart compress: preserve skill teach docs + recent conversation context
        smart_compress(&mut msgs, params.tool_frequency, 5, params.max_conversation_turns);
        return msgs;
    }

    // First turn: build from scratch
    let system_prompt = params.system_prompt_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| build_system_prompt(params.tool_index, params.hot_tools, params.skills, params.user_memory, params.user_profile, params.plan_then_execute));

    let mut msgs = vec![serde_json::json!({
        "role": "system",
        "content": system_prompt,
    })];

    // Inject reminder right after system prompt
    if let Some(rt) = params.reminder_text {
        inject_reminder(&mut msgs, rt);
    }

    // Keep last N display messages for context
    let max_turns = params.max_conversation_turns;
    let start = params.app_messages.len().saturating_sub(max_turns);

    for msg in &params.app_messages[start..] {
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

    msgs.push(serde_json::json!({"role": "user", "content": params.user_text}));
    msgs
}

/// Execute a parsed tool call and return the result.
/// Tries built-in tools first, then falls back to MCP-discovered tools.
#[tracing::instrument(skip(args, skills, ctx))]
pub(crate) fn execute_tool_call(
    name: &str,
    args: &Value,
    skills: &[SkillDefinition],
    mcp: Option<&McpRegistry>,
    ctx: &crate::tools::ToolContext,
) -> String {
    // Try built-in tools first (including skill tools)
    let registry = crate::tools::ToolRegistry::with_skills(skills);
    if registry.tool_exists(name) {
        // Tool found — let it execute; propagate real CLI error (not "unknown tool")
        return registry.execute(name, args, ctx).unwrap_or_else(|e| e.to_string());
    }

    // Try MCP-discovered tools (from the registry parameter)
    if let Some(mcp) = mcp {
        for (client_idx, tool_def) in &mcp.tools {
            if tool_def.name == name
                && let Some(client) = mcp.clients.get(*client_idx) {
                    return client.call_tool(name, args).unwrap_or_else(|e| format!("MCP 错误: {}", e));
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
    scored.sort_by_key(|&(score, _)| std::cmp::Reverse(score));
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
pub fn smart_compress(
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
    for &pos in sorted_ranks.iter().take(max_teach_docs) {
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
    // Use a single forward + backward scan (O(n)) instead of an O(n²) while-loop.
    // Forward: if a tool result is kept, ensure preceding tool_call is kept.
    for idx in 1..msgs.len() {
        if preserve.contains(&idx)
            && msgs[idx].get("role").and_then(|r| r.as_str()) == Some("tool")
            && msgs[idx - 1].get("tool_calls").is_some()
        {
            preserve.insert(idx - 1);
        }
    }
    // Backward: if a tool_call is kept, ensure following tool result is kept.
    for idx in (0..msgs.len().saturating_sub(1)).rev() {
        if preserve.contains(&idx)
            && msgs[idx].get("tool_calls").is_some()
            && msgs[idx + 1].get("role").and_then(|r| r.as_str()) == Some("tool")
            && !preserve.contains(&(idx + 1))
        {
            preserve.insert(idx + 1);
        }
    }

    // Build compressed message list
    let mut new_msgs: Vec<Value> = Vec::with_capacity(preserve.len());
    for (idx, msg) in msgs[..recent_start].iter().enumerate() {
        if preserve.contains(&idx) {
            new_msgs.push(msg.clone());
        }
    }
    for msg in msgs[recent_start..].iter() {
        new_msgs.push(msg.clone());
    }

    *msgs = new_msgs;
}

/// Compress API messages after a conversation turn completes.
///
/// Preserves top 5 skill teach docs (scored by cross-session frequency + recency)
/// and the last 20 conversation messages for context.
#[allow(dead_code)]
pub fn compress_api_messages(
    msgs: &mut Vec<Value>,
    tool_frequency: &HashMap<String, usize>,
) {
    smart_compress(msgs, tool_frequency, 5, 20);
}

/// Main chat loop: stream, handle tool calls, continue until done
#[tracing::instrument(skip(provider, config, messages, tx, mcp, skills))]
pub async fn chat_loop(
    provider: Box<dyn LlmProvider>,
    config: Config,
    messages: Vec<Value>,
    tx: mpsc::UnboundedSender<LlmEvent>,
    mcp: McpRegistry,
    skills: Vec<SkillDefinition>,
) {
    let enabled = if config.enabled_tools.is_empty() {
        None
    } else {
        Some(&config.enabled_tools)
    };
    let mut tool_schemas = crate::tools::ToolRegistry::with_skills(&skills).enabled_schemas(enabled);
    // Append MCP tool schemas if available
    for (client_idx, tool_def) in &mcp.tools {
        if let Some(_client) = mcp.clients.get(*client_idx) {
            let schema = crate::tools::mcp_tools::mcp_schema_to_openai(tool_def);
            tool_schemas.push(schema);
        }
    }
    let mut msgs = messages;
    let tool_ctx = crate::tools::ToolContext {
        config: config.clone(),
        mcp: mcp.clone(),
    };
    let mut retry_counts: HashMap<String, u32> = HashMap::new();
    let max_retries = config.max_tool_retries;
    let max_rounds = config.max_react_rounds;
    let mut round_count = 0u32;

    // Create shared ToolCallExecutor with configurable parameters
    let executor = crate::core::executor::ToolCallExecutor::new(tool_ctx, mcp.clone(), skills.clone())
        .with_timeout(config.cli_timeout_secs)
        .with_truncation(200, 500);

    loop {
        round_count += 1;
        if round_count > max_rounds {
            let _ = tx.send(LlmEvent::Error(format!("已达最大执行轮数限制 ({}), 已停止循环。", max_rounds)));
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
                let _ = tx.send(LlmEvent::Done(Arc::new(msgs), usage));
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

                // Parallel execute all tool calls via ToolCallExecutor
                let total = calls.len();
                let _ = tx.send(LlmEvent::Status(format!("⚡ 并行执行 {} 个工具...", total)));

                let all_results = executor.execute(calls, &tx).await;

                // Check for errors and track retry counts
                let mut should_retry = false;
                for result in &all_results {
                    if result.result.starts_with("错误:") {
                        let count = retry_counts.entry(result.call.id.clone()).or_insert(0);
                        *count += 1;
                        if *count <= max_retries {
                            should_retry = true;
                        }
                    }
                }

                if should_retry {
                    // Push all results so LLM sees what succeeded/failed
                    for result in &all_results {
                        msgs.push(serde_json::json!({ "role": "tool", "tool_call_id": result.call.id, "content": utils::smart_truncate(&result.result, 500) }));
                    }
                    // Add retry guidance
                    msgs.push(serde_json::json!({
                        "role": "system",
                        "content": "部分工具调用返回错误，请修正参数后重试。".to_string(),
                    }));
                } else {
                    // Add reflection for max-retries-exceeded errors, then push all results
                    for result in &all_results {
                        if result.result.starts_with("错误:") {
                            msgs.push(serde_json::json!({
                                "role": "system",
                                "content": format!(
                                    "工具 '{}' 连续 {} 次调用失败。请反思：\n\
                                     1. 参数是否正确？\n\
                                     2. 是否需要换一种方式完成用户请求？\n\
                                     3. 是否不需要这个工具，用其他方式回答用户？\n\
                                     错误信息：{}",
                                    result.call.name, max_retries, result.result
                                ),
                            }));
                        }
                        let trimmed = utils::smart_truncate(&result.result, 500);
                        msgs.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": result.call.id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ProviderKind;
    use serde_json::json;

    // ── smart_compress tests ──

    #[test]
    fn test_smart_compress_empty_noop() {
        let mut msgs = vec![];
        smart_compress(&mut msgs, &HashMap::new(), 5, 5);
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_smart_compress_below_threshold_noop() {
        let msgs = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "hi"}),
            json!({"role": "assistant", "content": "hello"}),
        ];
        // 3 messages <= 1 + recent_keep(5) = 6 → no-op
        let expected = msgs.clone();
        let mut actual = msgs;
        smart_compress(&mut actual, &HashMap::new(), 5, 5);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_smart_compress_keeps_system_message() {
        let mut msgs: Vec<Value> = (0..20).map(|i| {
            json!({"role": "user", "content": format!("msg {}", i)})
        }).collect();
        msgs.insert(0, json!({"role": "system", "content": "sys"}));
        smart_compress(&mut msgs, &HashMap::new(), 5, 5);
        assert_eq!(msgs[0]["role"], "system");
    }

    #[test]
    fn test_smart_compress_keeps_recent_messages() {
        let mut msgs: Vec<Value> = (0..20).map(|i| {
            json!({"role": "user", "content": format!("msg {}", i)})
        }).collect();
        msgs.insert(0, json!({"role": "system", "content": "sys"}));
        let before_len = msgs.len();
        smart_compress(&mut msgs, &HashMap::new(), 5, 5);
        assert!(msgs.len() < before_len);
        // Last 5 messages always kept
        assert_eq!(msgs[msgs.len() - 1]["content"], "msg 19");
        assert_eq!(msgs[msgs.len() - 5]["content"], "msg 15");
    }

    #[test]
    fn test_smart_compress_keeps_top_teach_pairs() {
        // Build messages with 2 teach pairs + some filler
        let mut msgs: Vec<Value> = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "filler 1"}),
            json!({"role": "assistant", "content": "filler 1 response"}),
            json!({"role": "user", "content": "filler 2"}),
            json!({"role": "assistant", "content": "filler 2 response"}),
        ];
        // Add teach pair for "weight" (high frequency = will be kept)
        msgs.push(json!({
            "role": "assistant",
            "tool_calls": [{
                "function": {
                    "name": "i_rs",
                    "arguments": r#"{"command":"skill","tool":"weight"}"#
                }
            }]
        }));
        msgs.push(json!({"role": "tool", "content": "weight skill doc"}));
        // Add teach pair for "mood" (low frequency = may be dropped)
        msgs.push(json!({
            "role": "assistant",
            "tool_calls": [{
                "function": {
                    "name": "i_rs",
                    "arguments": r#"{"command":"skill","tool":"mood"}"#
                }
            }]
        }));
        msgs.push(json!({"role": "tool", "content": "mood skill doc"}));
        // Add filler to push some beyond recent window
        for i in 0..10 {
            msgs.push(json!({"role": "user", "content": format!("recent {}", i)}));
            msgs.push(json!({"role": "assistant", "content": format!("response {}", i)}));
        }
        // Add user query at end
        msgs.push(json!({"role": "user", "content": "final query"}));

        let mut freq = HashMap::new();
        freq.insert("weight".to_string(), 5);
        freq.insert("mood".to_string(), 1);

        smart_compress(&mut msgs, &freq, 1, 5);

        // Weight teach pair (high freq) should be preserved
        let content_str = serde_json::to_string(&msgs).unwrap();
        assert!(content_str.contains("weight skill doc"), "high-frequency teach pair should be kept");
    }

    #[test]
    fn test_smart_compress_preserves_tool_call_pairs() {
        // Create messages where a tool_call is about to be dropped but tool result is kept
        let mut msgs: Vec<Value> = vec![
            json!({"role": "system", "content": "sys"}),
        ];
        // Add many filler messages that will be outside the recent window
        for i in 0..15 {
            msgs.push(json!({"role": "user", "content": format!("old msg {}", i)}));
            msgs.push(json!({"role": "assistant", "content": format!("old resp {}", i)}));
        }
        // Add a tool_call + tool result pair
        msgs.push(json!({
            "role": "assistant",
            "tool_calls": [{"function": {"name": "some_tool", "arguments": "{}"}}]
        }));
        msgs.push(json!({"role": "tool", "content": "result"}));
        // Recent messages
        msgs.push(json!({"role": "user", "content": "recent"}));
        msgs.push(json!({"role": "assistant", "content": "response"}));

        smart_compress(&mut msgs, &HashMap::new(), 5, 5);

        // The tool_call + tool result should be preserved as a pair
        let content_str = serde_json::to_string(&msgs).unwrap();
        assert!(content_str.contains("result"), "tool result should be kept");
        assert!(content_str.contains("some_tool"), "tool_call should be kept");
    }

    // ── build_messages tests ──

    #[test]
    fn test_build_messages_first_turn() {
        use crate::app::Message;
        let params = MessageBuildParams {
            app_messages: &[Message::User { text: "hello".to_string() }],
            user_text: "hello",
            saved_api_messages: &None,
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: None,
            system_prompt_override: Some("custom system prompt"),
            plan_then_execute: false,
            max_conversation_turns: 8,
        };
        let result = build_messages(params);
        // system + app_messages + user_text = 3
        assert_eq!(result.len(), 3);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[0]["content"], "custom system prompt");
        assert_eq!(result[1]["role"], "user");
        assert_eq!(result[1]["content"], "hello");
    }

    #[test]
    fn test_build_messages_second_turn() {
        use crate::app::Message;
        let saved = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "prev"}),
            json!({"role": "assistant", "content": "response"}),
        ];
        let params = MessageBuildParams {
            app_messages: &[Message::User { text: "prev".to_string() }, Message::Assistant { text: "response".to_string() }, Message::User { text: "new".to_string() }],
            user_text: "new",
            saved_api_messages: &Some(saved),
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: None,
            system_prompt_override: None,
            plan_then_execute: false,
            max_conversation_turns: 8,
        };
        let result = build_messages(params);
        // Should have: system + user(prev) + assistant(response) + user(new)
        assert!(result.len() >= 3);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[result.len() - 1]["role"], "user");
        assert_eq!(result[result.len() - 1]["content"], "new");
    }

    #[test]
    fn test_build_messages_with_reminder() {
        use crate::app::Message;
        let params = MessageBuildParams {
            app_messages: &[Message::User { text: "remind".to_string() }],
            user_text: "remind",
            saved_api_messages: &None,
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: Some("吃药"),
            system_prompt_override: Some("sys"),
            plan_then_execute: false,
            max_conversation_turns: 8,
        };
        let result = build_messages(params);
        // system + reminder + app_message + user_text = 4
        assert_eq!(result.len(), 4);
        assert_eq!(result[0]["role"], "system");
        assert_eq!(result[1]["role"], "system");
        assert!(result[1]["content"].as_str().unwrap().contains("吃药"));
        assert_eq!(result[2]["role"], "user");
    }

    #[test]
    fn test_build_messages_second_turn_stale_reminder_removed() {
        use crate::app::Message;
        // Simulate saved messages with old reminder at index 1
        let saved = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "system", "content": "注意：用户有以下即将到期或已到期的提醒事项：\n- 吃药"}),
            json!({"role": "user", "content": "done"}),
            json!({"role": "assistant", "content": "ok"}),
        ];
        let params = MessageBuildParams {
            app_messages: &[Message::User { text: "done".to_string() }, Message::Assistant { text: "ok".to_string() }, Message::User { text: "new".to_string() }],
            user_text: "new",
            saved_api_messages: &Some(saved),
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: Some("新提醒"),
            system_prompt_override: None,
            plan_then_execute: false,
            max_conversation_turns: 8,
        };
        let result = build_messages(params);
        // Old reminder should be replaced by new one
        let system_msgs: Vec<_> = result.iter().filter(|m| m["role"] == "system").collect();
        assert_eq!(system_msgs.len(), 2);  // original system + new reminder
        let has_old_reminder = system_msgs.iter().any(|m|
            m["content"].as_str().unwrap_or("").contains("吃药"));
        assert!(!has_old_reminder, "old reminder should be removed");
        let has_new_reminder = system_msgs.iter().any(|m|
            m["content"].as_str().unwrap_or("").contains("新提醒"));
        assert!(has_new_reminder, "new reminder should be present");
    }

    #[test]
    fn test_build_messages_max_turns() {
        use crate::app::Message;
        // Create 20 display messages but only keep last 2 turns
        let app_msgs: Vec<Message> = (0..20).flat_map(|i| vec![
            Message::User { text: format!("q{}", i) },
            Message::Assistant { text: format!("a{}", i) },
        ]).collect();
        let params = MessageBuildParams {
            app_messages: &app_msgs,
            user_text: "final",
            saved_api_messages: &None,
            tool_frequency: &HashMap::new(),
            tool_index: "",
            hot_tools: "",
            skills: "",
            user_memory: "",
            user_profile: "",
            reminder_text: None,
            system_prompt_override: Some("sys"),
            plan_then_execute: false,
            max_conversation_turns: 2,  // only last 2 turns = 4 messages
        };
        let result = build_messages(params);
        // system + last 2 app_messages + final user = 4
        assert_eq!(result.len(), 4);
        assert_eq!(result[1]["content"], "q19");
        assert_eq!(result[2]["content"], "a19");
        assert_eq!(result[3]["content"], "final");
    }

    // ── chat_loop 测试 ──

    /// 用于模拟始终返回 ToolCalls 的 provider
    struct AlwaysToolCall;

    #[async_trait::async_trait]
    impl LlmProvider for AlwaysToolCall {
        fn kind(&self) -> ProviderKind {
            ProviderKind::OpenAI
        }
        fn model(&self) -> &str {
            "mock"
        }
        async fn stream_chat(
            &self,
            _msgs: &[Value],
            _schemas: &[Value],
            _tx: &mpsc::UnboundedSender<LlmEvent>,
        ) -> anyhow::Result<StreamResult> {
            Ok(StreamResult::ToolCalls(Vec::new(), String::new()))
        }
    }

    #[tokio::test]
    async fn test_chat_loop_single_turn_text() {
        let provider: Box<dyn LlmProvider> = Box::new(
            crate::test_helpers::MockProvider::new(vec![LlmEvent::Token("hello".to_string())]),
        );
        let config = crate::test_helpers::test_config();
        let mcp = crate::mcp::McpRegistry::empty_for_test();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let messages = vec![json!({"role": "user", "content": "hi"})];

        chat_loop(provider, config, messages, tx, mcp, vec![]).await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Token(t) if t == "hello")),
            "应收到 Token 事件"
        );
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Done(..))),
            "应收到 Done 事件"
        );
    }

    #[tokio::test]
    async fn test_chat_loop_provider_error() {
        let provider: Box<dyn LlmProvider> = Box::new(
            crate::test_helpers::MockProvider::new(vec![])
                .with_result(Err(anyhow::anyhow!("模拟错误"))),
        );
        let config = crate::test_helpers::test_config();
        let mcp = crate::mcp::McpRegistry::empty_for_test();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let messages = vec![json!({"role": "user", "content": "hi"})];

        chat_loop(provider, config, messages, tx, mcp, vec![]).await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Error(msg) if msg.contains("模拟错误"))),
            "应收到 Error 事件"
        );
    }

    #[tokio::test]
    async fn test_chat_loop_max_rounds_exceeded() {
        let mut config = crate::test_helpers::test_config();
        config.max_react_rounds = 2;
        let mcp = crate::mcp::McpRegistry::empty_for_test();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let messages = vec![json!({"role": "user", "content": "do work"})];

        chat_loop(Box::new(AlwaysToolCall), config, messages, tx, mcp, vec![]).await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        assert!(
            events.iter().any(|e| matches!(e, LlmEvent::Error(_))),
            "超出最大轮次后应收到 Error 事件"
        );
    }

    #[test]
    fn test_plan_then_execute_prompt() {
        let prompt = build_system_prompt("", "", "", "", "", true);
        assert!(
            prompt.contains("Plan-then-Execute"),
            "plan_then_execute=true 时系统提示词应包含 Plan-then-Execute 模式说明"
        );
        assert!(
            prompt.contains("执行计划"),
            "应包含'执行计划'关键词"
        );
    }

    #[test]
    fn test_react_prompt_default() {
        let prompt = build_system_prompt("", "", "", "", "", false);
        assert!(
            prompt.contains("无需预先规划整个流程"),
            "plan_then_execute=false 时系统提示词应包含 ReAct 模式说明"
        );
    }

    #[test]
    fn test_build_system_prompt_date_injection() {
        let prompt = build_system_prompt("", "", "", "", "", false);
        // 注入当前日期，验证格式
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(prompt.contains(&today), "应注入当前日期");
        // {current_date} 占位符应已被替换
        assert!(!prompt.contains("{current_date}"), "占位符应被替换");
    }

    #[test]
    fn test_build_system_prompt_tool_index_injection() {
        let prompt = build_system_prompt("★工具索引★", "", "", "", "", false);
        assert!(prompt.contains("★工具索引★"), "应注入工具索引");
        assert!(!prompt.contains("{{TOOL_INDEX}}"), "TOOL_INDEX 占位符应被替换");
    }

    #[test]
    fn test_build_system_prompt_all_sections() {
        let prompt = build_system_prompt(
            "TOOLS",
            "HOT_TOOLS",
            "SKILLS",
            "MEMORY",
            "PROFILE",
            false,
        );
        assert!(prompt.contains("TOOLS"), "应有工具索引");
        assert!(prompt.contains("HOT_TOOLS"), "应有热门工具");
        assert!(prompt.contains("SKILLS"), "应有技能");
        assert!(prompt.contains("MEMORY"), "应有记忆");
        assert!(prompt.contains("PROFILE"), "应有用户画像");
        // 所有占位符应被替换
        assert!(!prompt.contains("{{TOOL_INDEX}}"));
        assert!(!prompt.contains("{{HOT_TOOLS}}"));
        assert!(!prompt.contains("{{SKILLS}}"));
        assert!(!prompt.contains("{{USER_MEMORY}}"));
        assert!(!prompt.contains("{{USER_PROFILE}}"));
        assert!(!prompt.contains("{{PLAN_MODE}}"));
    }
}

