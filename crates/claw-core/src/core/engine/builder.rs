use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::core::context::ContextManager;
use crate::error::category_from_result;

// ── System Prompt Layer ──

const REMINDER_PREFIX: &str = "注意：用户有以下即将到期或已到期的提醒事项";
const SUMMARY_PREFIX: &str = "[先前上下文摘要]";

const REACT_PROMPT: &str = "\
当用户请求涉及 **2 个或以上不同工具调用** 时：
1. 无需预先规划整个流程，直接开始执行第一步
2. 根据上一步的结果自然决定下一步
3. 按顺序依次执行，每次调用一个工具
4. 所有步骤完成后给出总结
";

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

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_system_prompt(
    tool_index: &str,
    hot_tools: &str,
    skills: &str,
    user_memory: &str,
    user_profile: &str,
    plan_then_execute: bool,
    tz_offset: chrono::FixedOffset,
    identity: &str,
    routing_hint: &str,
) -> String {
    let template = include_str!("../../../prompts/system.md");
    let now = crate::utils::now_in_tz(tz_offset);
    let today = now.format("%Y-%m-%d").to_string();
    let weekday = now.format("%A").to_string();
    let time_str = now.format("%H:%M").to_string();
    let tz_label = crate::utils::tz_label(tz_offset);

    let plan_mode = if plan_then_execute {
        PLAN_THEN_EXECUTE_PROMPT
    } else {
        REACT_PROMPT
    };

    let routing_hint_val = if routing_hint.is_empty() {
        ""
    } else {
        routing_hint
    };

    let replacements: [(&str, &str); 12] = [
        ("{current_date}", &today),
        ("{current_weekday}", &weekday),
        ("{current_time}", &time_str),
        ("{timezone}", &tz_label),
        ("{{PLAN_MODE}}", plan_mode),
        ("{{TOOL_INDEX}}", tool_index),
        ("{{IDENTITY}}", identity),
        ("{{HOT_TOOLS}}", hot_tools),
        ("{{SKILLS}}", skills),
        ("{{USER_MEMORY}}", user_memory),
        ("{{USER_PROFILE}}", user_profile),
        ("{{ROUTING_HINT}}", routing_hint_val),
    ];

    let estimated_len = template.len()
        + tool_index.len()
        + hot_tools.len()
        + skills.len()
        + user_memory.len()
        + user_profile.len();
    let mut result = String::with_capacity(estimated_len);

    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut matched = false;
            for (pattern, value) in &replacements {
                if template[i..].starts_with(*pattern) {
                    result.push_str(value);
                    i += pattern.len();
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
        }
        let c = template[i..].chars().next().unwrap();
        result.push(c);
        i += c.len_utf8();
    }

    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    result
}

// ── Message Building ──

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
    pub routing_hint: &'a str,
    /// Model identifier for ContextManager token sizing.
    pub model: &'a str,
}

pub fn build_messages(params: MessageBuildParams) -> Vec<Value> {
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

    let ctx_mgr = ContextManager::for_model(params.model);

    if let Some(prev_msgs) = params.saved_api_messages {
        let mut msgs = prev_msgs.clone();
        remove_reminder_msg(&mut msgs);
        if msgs.len() > 1
            && msgs
                .last()
                .and_then(|m| m.get("role").and_then(|r| r.as_str()))
                == Some("user")
        {
            msgs.pop();
        }
        msgs.push(serde_json::json!({"role": "user", "content": params.user_text}));

        if let Some(rt) = params.reminder_text {
            inject_reminder(&mut msgs, rt);
        }

        // Inject structural summary for old context before compression
        if let Some(summary) = ctx_mgr.structural_summary(&msgs, params.max_conversation_turns) {
            remove_reminder_msg(&mut msgs);
            let mut insert_pos = 1;
            if msgs.len() > 1
                && msgs[1].get("role").and_then(|r| r.as_str()) == Some("system")
                && msgs[1]
                    .get("content")
                    .and_then(|c| c.as_str())
                    .is_some_and(|c| c.starts_with(REMINDER_PREFIX))
            {
                insert_pos = 2;
            }
            msgs.insert(
                insert_pos,
                serde_json::json!({
                    "role": "system",
                    "content": summary,
                }),
            );
        }

        ctx_mgr.compress(&mut msgs, params.tool_frequency);
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
                params.routing_hint,
            )
        });

    let mut msgs = vec![serde_json::json!({
        "role": "system",
        "content": system_prompt,
    })];

    if let Some(rt) = params.reminder_text {
        inject_reminder(&mut msgs, rt);
    }

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

    let last_is_current_user = msgs
        .last()
        .and_then(|m| m.get("content").and_then(|c| c.as_str()))
        == Some(params.user_text);
    if !last_is_current_user {
        msgs.push(serde_json::json!({"role": "user", "content": params.user_text}));
    }

    // First turn compression: guard against oversized system prompt + messages
    if msgs.len() > ctx_mgr.min_retain + 2 {
        ctx_mgr.compress(&mut msgs, params.tool_frequency);
    }

    msgs
}

// ── Semantic Message Scoring ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageSignificance {
    /// System prompt — never drop
    System,
    /// User correction / negation — keep at all cost
    Correction,
    /// Decision / confirmation — high priority
    Decision,
    /// Tool result with meaningful data — high priority
    ToolData,
    /// Normal dialogue — medium priority
    Dialogue,
    /// Greeting / acknowledgment / error — low priority
    LowValue,
}

/// Score a message for retention priority. Higher score = more important.
fn score_message_significance(msg: &Value) -> (MessageSignificance, u8) {
    let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");

    let content_val = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");

    if role == "system" {
        if content_val.starts_with(SUMMARY_PREFIX) {
            return (MessageSignificance::System, 10);
        }
        return (MessageSignificance::System, 10);
    }

    match role {
        "user" => {
            if crate::utils::is_correction_message(content_val) {
                (MessageSignificance::Correction, 9)
            } else if crate::utils::is_decision_message(content_val) {
                (MessageSignificance::Decision, 7)
            } else if content_val.len() < 6 {
                (MessageSignificance::LowValue, 1)
            } else {
                (MessageSignificance::Dialogue, 4)
            }
        }
        "assistant" => {
            if crate::utils::is_decision_message(content_val) {
                (MessageSignificance::Decision, 7)
            } else if content_val.len() < 10 {
                (MessageSignificance::LowValue, 1)
            } else {
                (MessageSignificance::Dialogue, 3)
            }
        }
        "tool" => {
            let cat = category_from_result(content_val);
            if cat.is_retryable_or_fatal() {
                (MessageSignificance::LowValue, 2)
            } else if content_val.len() > 20 {
                (MessageSignificance::ToolData, 6)
            } else {
                (MessageSignificance::LowValue, 1)
            }
        }
        _ => (MessageSignificance::Dialogue, 2),
    }
}

// ── Smart Compression ──

struct TeachPair {
    assist_idx: usize,
    result_idx: usize,
    tool_name: String,
}

fn find_teach_pairs(msgs: &[Value]) -> Vec<TeachPair> {
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

/// Score teach pairs by global cross-session frequency * weight + semantic import + recency.
fn score_teach_pairs(pairs: &[TeachPair], tool_frequency: &HashMap<String, usize>) -> Vec<usize> {
    let max_recency = pairs.len().max(1);
    let mut scored: Vec<(usize, usize)> = pairs
        .iter()
        .enumerate()
        .map(|(pos, pair)| {
            let freq = tool_frequency.get(&pair.tool_name).copied().unwrap_or(0);
            let recency = max_recency - pos;
            (freq * 15 + recency, pos)
        })
        .collect();
    scored.sort_by_key(|&(score, _)| std::cmp::Reverse(score));
    scored.into_iter().map(|(_, pos)| pos).collect()
}

/// Smart compress API message list, preserving high-value content.
///
/// Strategy:
/// 1. Always keep system message (index 0)
/// 2. Score teach doc pairs by cross-session frequency + recency
/// 3. Score regular messages by semantic significance (corrections, decisions, data)
/// 4. Keep top-scoring teach pairs + high-semantic-value messages outside recent window
/// 5. Keep recent conversation messages intact
/// 6. Guarantee min_retain floor
pub fn smart_compress(
    msgs: &mut Vec<Value>,
    tool_frequency: &HashMap<String, usize>,
    max_teach_docs: usize,
    recent_keep: usize,
    min_retain: usize,
) {
    let effective_recent = recent_keep.max(min_retain);
    if msgs.len() <= 1 + effective_recent {
        return;
    }

    let teach_pairs = find_teach_pairs(msgs);
    let sorted_ranks = score_teach_pairs(&teach_pairs, tool_frequency);

    let mut preserve: HashSet<usize> = HashSet::new();
    preserve.insert(0);

    // Keep top-scoring teach pairs
    for &pos in sorted_ranks.iter().take(max_teach_docs) {
        preserve.insert(teach_pairs[pos].assist_idx);
        preserve.insert(teach_pairs[pos].result_idx);
    }

    // Score and preserve semantically important messages outside the recent window
    let recent_start = msgs.len().saturating_sub(effective_recent);
    for (idx, msg) in msgs.iter().enumerate().take(recent_start).skip(1) {
        let (sig, score) = score_message_significance(msg);
        if score >= 7 {
            // Correction, Decision, ToolData — always keep
            preserve.insert(idx);
        } else if score >= 5 && sig == MessageSignificance::ToolData {
            preserve.insert(idx);
        }
    }

    // Keep recent conversation messages
    for idx in recent_start..msgs.len() {
        preserve.insert(idx);
    }

    // Ensure tool_call + tool result pairs are kept together
    for idx in 1..msgs.len() {
        if preserve.contains(&idx)
            && msgs[idx].get("role").and_then(|r| r.as_str()) == Some("tool")
            && msgs[idx - 1].get("tool_calls").is_some()
        {
            preserve.insert(idx - 1);
        }
    }
    for idx in 0..msgs.len() {
        if preserve.contains(&idx) && msgs[idx].get("tool_calls").is_some() {
            let mut j = idx + 1;
            while j < msgs.len() && msgs[j].get("role").and_then(|r| r.as_str()) == Some("tool") {
                preserve.insert(j);
                j += 1;
            }
        }
    }

    // Build compressed list, honoring min_retain floor
    let mut new_msgs: Vec<Value> = Vec::with_capacity(preserve.len());
    for (idx, msg) in msgs[..recent_start].iter().enumerate() {
        if preserve.contains(&idx) {
            new_msgs.push(msg.clone());
        }
    }
    for msg in msgs[recent_start..].iter() {
        new_msgs.push(msg.clone());
    }

    if new_msgs.len() < min_retain {
        // Floor safeguard: keep last min_retain messages from original
        let keep_start = msgs.len().saturating_sub(min_retain);
        let mut floor = Vec::with_capacity(min_retain + 1);
        floor.push(msgs[0].clone()); // system
        for msg in &msgs[keep_start..] {
            floor.push(msg.clone());
        }
        *msgs = floor;
    } else {
        *msgs = new_msgs;
    }
}

#[allow(dead_code)]
pub fn compress_api_messages(msgs: &mut Vec<Value>, tool_frequency: &HashMap<String, usize>) {
    smart_compress(msgs, tool_frequency, 5, 12, 6);
}
