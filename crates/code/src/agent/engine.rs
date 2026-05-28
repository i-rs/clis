use crate::provider::*;
use crate::tools::ToolRegistry;
use super::event::AgentEvent;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;

const MAX_PROVIDER_RETRIES: u32 = 2;
const MAX_TOOL_RETRIES: u32 = 2;
const DEFAULT_TOOL_TIMEOUT_SECS: u64 = 120;

fn is_transient_error(e: &str) -> bool {
    let lower = e.to_lowercase();
    lower.contains("限流")
        || lower.contains("rate")
        || lower.contains("timeout")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("连接")
        || lower.contains("connection")
        || lower.contains("econnreset")
        || lower.contains("econnrefused")
}

struct ToolExecResult {
    tool_messages: Vec<(String, String, String)>,
}

async fn execute_tools(
    pending_tool_calls: &[ToolCall],
    tools: &ToolRegistry,
    retry_counts: &mut HashMap<String, u32>,
    tool_timeout_secs: u64,
) -> ToolExecResult {
    let handles: Vec<_> = pending_tool_calls.iter().map(|tc| {
        let tool = tools.get(&tc.name);
        let tc = ToolCall { id: tc.id.clone(), name: tc.name.clone(), args: tc.args.clone() };
        tokio::spawn(async move {
            let result = if let Some(tool) = tool {
                match tc.args.as_object() {
                    Some(obj) => tool.call(obj).await,
                    None => tool.call(&serde_json::Map::new()).await,
                }
            } else {
                Err(anyhow::anyhow!("Unknown tool: {}", tc.name))
            };
            (tc, result)
        })
    }).collect();

    let tc_list: Vec<ToolCall> = pending_tool_calls.to_vec();
    let mut tool_messages = Vec::new();

    for (i, handle) in handles.into_iter().enumerate() {
        let tc = &tc_list[i];
        let result_str = match tokio::time::timeout(std::time::Duration::from_secs(tool_timeout_secs), handle).await {
            Ok(Ok(inner)) => match &inner.1 {
                Ok(s) => s.clone(),
                Err(e) => format!("Error: {}", e),
            },
            Ok(Err(join_err)) => {
                format!("Error: tool task panicked: {}", join_err)
            }
            Err(_) => {
                format!("Error: tool execution timed out ({}s)", tool_timeout_secs)
            }
        };

        if result_str.starts_with("Error:") {
            let count = retry_counts.entry(tc.id.clone()).or_insert(0);
            *count += 1;
        } else {
            retry_counts.remove(&tc.id);
        }

        tool_messages.push((tc.name.clone(), tc.id.clone(), result_str));
    }

    ToolExecResult { tool_messages }
}

fn build_over_limit_message(retry_counts: &HashMap<String, u32>) -> Option<String> {
    let over_limit: Vec<_> = retry_counts.iter()
        .filter(|(_, c)| **c > MAX_TOOL_RETRIES)
        .map(|(id, c)| (id.clone(), *c))
        .collect();
    if over_limit.is_empty() {
        return None;
    }
    let details: Vec<String> = over_limit.iter()
        .map(|(id, c)| format!("tool_call_id='{}' ({}次)", id, c))
        .collect();
    Some(format!(
        "工具连续 {} 次调用失败。请反思：\n1. 参数是否正确？\n2. 是否需要换一种方式？\n3. 是否不需要这个工具？\n失败的调用: {}",
        MAX_TOOL_RETRIES,
        details.join("; ")
    ))
}

pub async fn react_loop(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    mut messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    json_output: bool,
    max_rounds: u32,
    tool_timeout_secs: u64,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let mut final_text = String::new();
    let mut total_usage = crate::provider::Usage { input_tokens: 0, output_tokens: 0 };
    let ctx = super::context::ContextManager::new();
    let mut provider_errors: u32 = 0;
    let mut retry_counts: HashMap<String, u32> = HashMap::new();

    for _round in 0..max_rounds {
        if ctx.should_compress(&messages) {
            messages = ctx.compress(&messages);
        }
        let mut rx = provider.stream(&messages, tool_defs).await;
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut pending_tool_calls = Vec::new();
        let mut round_usage: Option<crate::provider::Usage> = None;
        let mut had_error = false;

        while let Some(event) = rx.recv().await {
            match event.kind {
                StreamEventKind::Token(t) => {
                    content.push_str(&t);
                    if json_output {
                        let event = serde_json::json!({"event": "token", "content": t});
                        println!("{}", serde_json::to_string(&event)?);
                    } else {
                        print!("{}", t);
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                }
                StreamEventKind::Reasoning(r) => {
                    reasoning.push_str(&r);
                }
                StreamEventKind::ToolCall { id, name, args } => {
                    pending_tool_calls.push(ToolCall { id, name, args });
                }
                StreamEventKind::Done { usage, .. } => {
                    round_usage = usage;
                    break;
                }
                StreamEventKind::Error(e) => {
                    had_error = true;
                    if is_transient_error(&e) && provider_errors < MAX_PROVIDER_RETRIES {
                        provider_errors += 1;
                        let wait = 3 * provider_errors as u64;
                        if json_output {
                            let ev = serde_json::json!({"event": "retry", "wait": wait, "attempt": provider_errors});
                            println!("{}", serde_json::to_string(&ev)?);
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                        continue;
                    }
                    anyhow::bail!("{}", e);
                }
            }
        }

        if had_error {
            continue;
        }

        provider_errors = 0;

        if let Some(u) = round_usage {
            total_usage.input_tokens = total_usage.input_tokens.saturating_add(u.input_tokens);
            total_usage.output_tokens = total_usage.output_tokens.saturating_add(u.output_tokens);
        }

        if !content.is_empty() || !reasoning.is_empty() || !pending_tool_calls.is_empty() {
            final_text = content.clone();
            if !pending_tool_calls.is_empty() {
                messages.push(LlmMessage::AssistantWithReasoning {
                    content,
                    reasoning,
                    tool_calls: pending_tool_calls.clone(),
                });
            } else if reasoning.is_empty() {
                messages.push(LlmMessage::Assistant(content));
            } else {
                messages.push(LlmMessage::AssistantWithReasoning { content, reasoning, tool_calls: Vec::new() });
            }
        }

        if pending_tool_calls.is_empty() {
            break;
        }

        let ToolExecResult { tool_messages } = execute_tools(&pending_tool_calls, tools, &mut retry_counts, tool_timeout_secs).await;

        for (name, call_id, result_str) in &tool_messages {
            if let Ok(val) = serde_json::from_str::<Value>(result_str)
                && val.get("requires_claw").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if json_output {
                        let event = serde_json::json!({
                            "event": "request",
                            "type": val.get("request_type"),
                            "content": val.get("content"),
                        });
                        println!("{}", serde_json::to_string(&event)?);
                    }
                    return Ok((val.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(), messages));
                }
            if let Ok(val) = serde_json::from_str::<Value>(result_str)
                && val.get("requires_registration").and_then(|v| v.as_bool()).unwrap_or(false)
                && let Some(tool_info) = val.get("tool")
                && json_output {
                    let event = serde_json::json!({
                        "event": "tool_created",
                        "tool": tool_info,
                    });
                    println!("{}", serde_json::to_string(&event)?);
            }
            if json_output {
                let event = serde_json::json!({
                    "event": "tool_result",
                    "tool_call_id": call_id,
                    "name": name,
                    "result": result_str,
                });
                println!("{}", serde_json::to_string(&event)?);
            }
            messages.push(LlmMessage::Tool {
                name: name.clone(),
                content: result_str.clone(),
                call_id: call_id.clone(),
            });
        }

        if let Some(sys_msg) = build_over_limit_message(&retry_counts) {
            messages.push(LlmMessage::System(sys_msg));
        }
    }

    Ok((final_text, messages))
}

pub async fn react_loop_streaming(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    mut messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    event_tx: mpsc::Sender<AgentEvent>,
    max_rounds: u32,
    tool_timeout_secs: u64,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let mut final_text = String::new();
    let mut total_usage = crate::provider::Usage { input_tokens: 0, output_tokens: 0 };
    let ctx = super::context::ContextManager::new();
    let mut provider_errors: u32 = 0;
    let mut retry_counts: HashMap<String, u32> = HashMap::new();

    for _round in 0..max_rounds {
        if ctx.should_compress(&messages) {
            messages = ctx.compress(&messages);
        }
        let mut rx = provider.stream(&messages, tool_defs).await;
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut pending_tool_calls = Vec::new();
        let mut round_usage: Option<crate::provider::Usage> = None;
        let mut had_error = false;

        while let Some(event) = rx.recv().await {
            match event.kind {
                StreamEventKind::Token(t) => {
                    content.push_str(&t);
                    if event_tx.send(AgentEvent::Token(t)).await.is_err() {
                        break;
                    }
                }
                StreamEventKind::Reasoning(r) => {
                    reasoning.push_str(&r);
                    if event_tx.send(AgentEvent::Reasoning(r)).await.is_err() {
                        break;
                    }
                }
                StreamEventKind::ToolCall { id, name, args } => {
                    pending_tool_calls.push(ToolCall { id: id.clone(), name: name.clone(), args: args.clone() });
                    if event_tx.send(AgentEvent::ToolCallStart {
                        id: id.clone(),
                        name,
                        args,
                    }).await.is_err() {
                        break;
                    }
                }
                StreamEventKind::Done { usage, .. } => {
                    round_usage = usage;
                    break;
                }
                StreamEventKind::Error(e) => {
                    had_error = true;
                    if is_transient_error(&e) && provider_errors < MAX_PROVIDER_RETRIES {
                        provider_errors += 1;
                        let wait = 3 * provider_errors as u64;
                        event_tx.send(AgentEvent::Status(format!("网络波动，{}s 后重试 ({}/{})...", wait, provider_errors, MAX_PROVIDER_RETRIES))).await.ok();
                        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                        continue;
                    }
                    event_tx.send(AgentEvent::Error(e.clone())).await.ok();
                    anyhow::bail!("{}", e);
                }
            }
        }

        if had_error {
            continue;
        }

        provider_errors = 0;

        if let Some(u) = round_usage {
            total_usage.input_tokens = total_usage.input_tokens.saturating_add(u.input_tokens);
            total_usage.output_tokens = total_usage.output_tokens.saturating_add(u.output_tokens);
        }

        if !content.is_empty() || !reasoning.is_empty() || !pending_tool_calls.is_empty() {
            final_text = content.clone();
            if !pending_tool_calls.is_empty() {
                messages.push(LlmMessage::AssistantWithReasoning {
                    content,
                    reasoning,
                    tool_calls: pending_tool_calls.clone(),
                });
            } else if reasoning.is_empty() {
                messages.push(LlmMessage::Assistant(content));
            } else {
                messages.push(LlmMessage::AssistantWithReasoning { content, reasoning, tool_calls: Vec::new() });
            }
        }

        if pending_tool_calls.is_empty() {
            break;
        }

        let ToolExecResult { tool_messages } = execute_tools(&pending_tool_calls, tools, &mut retry_counts, tool_timeout_secs).await;

        for (name, call_id, result_str) in &tool_messages {
            event_tx.send(AgentEvent::ToolCallEnd {
                id: call_id.clone(),
                name: name.clone(),
                result: result_str.clone(),
            }).await.ok();

            if let Ok(val) = serde_json::from_str::<Value>(result_str)
                && val.get("requires_claw").and_then(|v| v.as_bool()).unwrap_or(false) {
                    event_tx.send(AgentEvent::Done { usage: Some(total_usage.clone()), messages: messages.clone(), context_pct: 0.0 }).await.ok();
                    return Ok((val.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(), messages));
                }

            messages.push(LlmMessage::Tool {
                name: name.clone(),
                content: result_str.clone(),
                call_id: call_id.clone(),
            });
        }

        if let Some(sys_msg) = build_over_limit_message(&retry_counts) {
            messages.push(LlmMessage::System(sys_msg));
        }
    }

    let _estimated = super::context::ContextManager::estimate_tokens(&messages) as f64 / 128_000.0;
    event_tx.send(AgentEvent::Done { usage: Some(total_usage), messages: messages.clone(), context_pct: _estimated }).await.ok();
    Ok((final_text, messages))
}
