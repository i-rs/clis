use serde_json::Value;
use std::collections::HashMap;

// ── System Prompt Layer ──

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
    tz_offset: chrono::FixedOffset,
    identity: &str,
) -> String {
    let mut prompt = include_str!("../../../prompts/system.md").to_string();
    let now = crate::utils::now_in_tz(tz_offset);
    let today = now.format("%Y-%m-%d").to_string();
    let weekday = now.format("%A").to_string();
    let time_str = now.format("%H:%M").to_string();
    let tz_label = crate::utils::tz_label(tz_offset);
    prompt = prompt
        .replace("{current_date}", &today)
        .replace("{current_weekday}", &weekday)
        .replace("{current_time}", &time_str)
        .replace("{timezone}", &tz_label);

    let plan_mode = if plan_then_execute {
        PLAN_THEN_EXECUTE_PROMPT
    } else {
        REACT_PROMPT
    };
    prompt = prompt.replace("{{PLAN_MODE}}", plan_mode);

    prompt = prompt.replace("{{TOOL_INDEX}}", tool_index);
    prompt = prompt.replace("{{IDENTITY}}", identity);
    prompt = prompt.replace("{{HOT_TOOLS}}", hot_tools);
    prompt = prompt.replace("{{SKILLS}}", skills);
    prompt = prompt.replace("{{USER_MEMORY}}", user_memory);
    prompt = prompt.replace("{{USER_PROFILE}}", user_profile);

    // Collapse 3+ consecutive newlines into 2 (one blank line)
    prompt = prompt.replace("\n\n\n\n", "\n\n");
    prompt = prompt.replace("\n\n\n", "\n\n");

    prompt
}

// ── Message Building ──

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
    pub tz_offset: chrono::FixedOffset,
    pub identity: &'a str,
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
        smart_compress(
            &mut msgs,
            params.tool_frequency,
            5,
            params.max_conversation_turns,
        );
        return msgs;
    }

    // First turn: build from scratch
    let system_prompt = params
        .system_prompt_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            build_system_prompt(
                params.tool_index,
                params.hot_tools,
                params.skills,
                params.user_memory,
                params.user_profile,
                params.plan_then_execute,
                params.tz_offset,
                params.identity,
            )
        });

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
            crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                msgs.push(serde_json::json!({"role": "assistant", "content": text}));
            }
            crate::app::Message::Evaluation {
                tool,
                valid,
                issues,
            } if !valid => {
                tracing::info!(tool, issues = ?issues, "工具结果验证告警");
            }
            _ => {}
        }
    }

    msgs.push(serde_json::json!({"role": "user", "content": params.user_text}));
    msgs
}

// ── Smart Compression ──

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
                if let Some(name) = tc
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                {
                    if name != "i_rs" {
                        continue;
                    }
                    let args_str = tc
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|a| a.as_str())
                        .unwrap_or("");
                    if let Ok(parsed) = serde_json::from_str::<Value>(args_str) {
                        let cmd = parsed.get("command").and_then(|c| c.as_str()).unwrap_or("");
                        if cmd != "skill" {
                            continue;
                        }
                        let tool_name = parsed
                            .get("tool")
                            .and_then(|t| t.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        if seen_tools.contains(&tool_name) {
                            continue;
                        }
                        seen_tools.insert(tool_name.clone());

                        if i + 1 < msgs.len()
                            && msgs[i + 1].get("role").and_then(|r| r.as_str()) == Some("tool")
                        {
                            pairs.push(TeachPair {
                                assist_idx: i,
                                result_idx: i + 1,
                                tool_name,
                            });
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
    let mut scored: Vec<(usize, usize)> = pairs
        .iter()
        .enumerate()
        .map(|(pos, pair)| {
            let freq = tool_frequency.get(&pair.tool_name).copied().unwrap_or(0);
            let recency = max_recency - pos;
            (freq * 10 + recency, pos)
        })
        .collect();
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
    // Backward: if a tool_call is kept, ensure ALL following tool results are kept.
    for idx in 0..msgs.len() {
        if preserve.contains(&idx) && msgs[idx].get("tool_calls").is_some() {
            let mut j = idx + 1;
            while j < msgs.len() && msgs[j].get("role").and_then(|r| r.as_str()) == Some("tool") {
                preserve.insert(j);
                j += 1;
            }
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
pub fn compress_api_messages(msgs: &mut Vec<Value>, tool_frequency: &HashMap<String, usize>) {
    smart_compress(msgs, tool_frequency, 5, 20);
}
