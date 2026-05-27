use crate::provider::*;
use crate::tools::ToolRegistry;
use super::event::AgentEvent;
use serde_json::Value;
use tokio::sync::mpsc;

pub async fn react_loop(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    mut messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    json_output: bool,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let max_rounds = 20;
    let mut final_text = String::new();
    let mut total_usage = crate::provider::Usage { input_tokens: 0, output_tokens: 0 };
    let ctx = super::context::ContextManager::new();

    for _round in 0..max_rounds {
        if ctx.should_compress(&messages) {
            messages = ctx.compress(&messages);
        }
        let mut rx = provider.stream(&messages, tool_defs).await;
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut pending_tool_calls = Vec::new();
        let mut round_usage: Option<crate::provider::Usage> = None;

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
                    if json_output {
                        let event = serde_json::json!({"event": "error", "content": e});
                        println!("{}", serde_json::to_string(&event)?);
                    }
                    anyhow::bail!("{}", e);
                }
            }
        }

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

        let tc_list: Vec<ToolCall> = pending_tool_calls.clone();

        for (i, handle) in handles.into_iter().enumerate() {
            let tc = &tc_list[i];
            let result = match tokio::time::timeout(std::time::Duration::from_secs(120), handle).await {
                Ok(Ok(inner)) => inner,
                Ok(Err(join_err)) => {
                    let result_str = format!("Error: tool task panicked: {}", join_err);
                    messages.push(LlmMessage::Tool {
                        name: tc.name.clone(),
                        content: result_str,
                        call_id: tc.id.clone(),
                    });
                    continue;
                }
                Err(_) => {
                    let result_str = "Error: tool execution timed out (120s)".to_string();
                    if json_output {
                        let event = serde_json::json!({
                            "event": "tool_result",
                            "tool_call_id": tc.id,
                            "name": tc.name,
                            "result": result_str,
                        });
                        println!("{}", serde_json::to_string(&event)?);
                    }
                    messages.push(LlmMessage::Tool {
                        name: tc.name.clone(),
                        content: result_str,
                        call_id: tc.id.clone(),
                    });
                    continue;
                }
            };

            let result_str = match &result.1 {
                Ok(s) => s.clone(),
                Err(e) => format!("Error: {}", e),
            };

            if let Ok(s) = &result.1
                && let Ok(val) = serde_json::from_str::<Value>(s) {
                    if val.get("requires_claw").and_then(|v| v.as_bool()).unwrap_or(false) {
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
                    if val.get("requires_registration").and_then(|v| v.as_bool()).unwrap_or(false)
                        && let Some(tool_info) = val.get("tool")
                        && json_output {
                            let event = serde_json::json!({
                                "event": "tool_created",
                                "tool": tool_info,
                            });
                            println!("{}", serde_json::to_string(&event)?);
                        }
                }

            if json_output {
                let event = serde_json::json!({
                    "event": "tool_result",
                    "tool_call_id": tc.id,
                    "name": tc.name,
                    "result": result_str,
                });
                println!("{}", serde_json::to_string(&event)?);
            }

            messages.push(LlmMessage::Tool {
                name: tc.name.clone(),
                content: result_str,
                call_id: tc.id.clone(),
            });
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
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let max_rounds = 20;
    let mut final_text = String::new();
    let mut total_usage = crate::provider::Usage { input_tokens: 0, output_tokens: 0 };
    let ctx = super::context::ContextManager::new();

    for _round in 0..max_rounds {
        if ctx.should_compress(&messages) {
            messages = ctx.compress(&messages);
        }
        let mut rx = provider.stream(&messages, tool_defs).await;
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut pending_tool_calls = Vec::new();
        let mut round_usage: Option<crate::provider::Usage> = None;

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
                    event_tx.send(AgentEvent::Error(e.clone())).await.ok();
                    anyhow::bail!("{}", e);
                }
            }
        }

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

        let tc_list: Vec<ToolCall> = pending_tool_calls.clone();

        for (i, handle) in handles.into_iter().enumerate() {
            let tc = &tc_list[i];
            let result = match tokio::time::timeout(std::time::Duration::from_secs(120), handle).await {
                Ok(Ok(inner)) => inner,
                Ok(Err(join_err)) => {
                    let result_str = format!("Error: tool task panicked: {}", join_err);
                    messages.push(LlmMessage::Tool {
                        name: tc.name.clone(),
                        content: result_str,
                        call_id: tc.id.clone(),
                    });
                    continue;
                }
                Err(_) => {
                    let result_str = "Error: tool execution timed out (120s)".to_string();
                    event_tx.send(AgentEvent::ToolCallEnd {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        result: result_str.clone(),
                    }).await.ok();
                    messages.push(LlmMessage::Tool {
                        name: tc.name.clone(),
                        content: result_str,
                        call_id: tc.id.clone(),
                    });
                    continue;
                }
            };

            let result_str = match &result.1 {
                Ok(s) => s.clone(),
                Err(e) => format!("Error: {}", e),
            };

            event_tx.send(AgentEvent::ToolCallEnd {
                id: tc.id.clone(),
                name: tc.name.clone(),
                result: result_str.clone(),
            }).await.ok();

            if let Ok(s) = &result.1
                && let Ok(val) = serde_json::from_str::<Value>(s)
                && val.get("requires_claw").and_then(|v| v.as_bool()).unwrap_or(false) {
                    event_tx.send(AgentEvent::Done { usage: Some(total_usage.clone()), messages: messages.clone() }).await.ok();
                    return Ok((val.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(), messages));
                }

            messages.push(LlmMessage::Tool {
                name: tc.name.clone(),
                content: result_str,
                call_id: tc.id.clone(),
            });
        }
    }

    event_tx.send(AgentEvent::Done { usage: Some(total_usage), messages: messages.clone() }).await.ok();
    Ok((final_text, messages))
}
