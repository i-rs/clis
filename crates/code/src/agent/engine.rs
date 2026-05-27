use crate::provider::*;
use crate::tools::ToolRegistry;
use serde_json::Value;

pub async fn react_loop(
    provider: &dyn LlmProvider,
    tools: &ToolRegistry,
    mut messages: Vec<LlmMessage>,
    tool_defs: &[Value],
    json_output: bool,
) -> anyhow::Result<(String, Vec<LlmMessage>)> {
    let max_rounds = 20;
    let mut final_text = String::new();

    for _round in 0..max_rounds {
        let mut rx = provider.stream(&messages, tool_defs).await;
        let mut content = String::new();
        let mut pending_tool_calls = Vec::new();

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
                StreamEventKind::ToolCall { id, name, args } => {
                    pending_tool_calls.push(ToolCall { id: id.clone(), name: name.clone(), args: args.clone() });
                    if json_output {
                        let event = serde_json::json!({"event": "tool_call", "name": name, "args": args});
                        println!("{}", serde_json::to_string(&event)?);
                    }
                }
                StreamEventKind::Done { .. } => break,
                StreamEventKind::Error(e) => {
                    if json_output {
                        let event = serde_json::json!({"event": "error", "content": e});
                        println!("{}", serde_json::to_string(&event)?);
                    }
                    anyhow::bail!("{}", e);
                }
            }
        }

        if !content.is_empty() {
            final_text = content.clone();
            messages.push(LlmMessage::Assistant(content));
        }

        if pending_tool_calls.is_empty() {
            break;
        }

        // Add tool calls to messages
        for tc in &pending_tool_calls {
            messages.push(LlmMessage::ToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                args: tc.args.clone(),
            });
        }

        // Execute tool calls in parallel
        for tc in &pending_tool_calls {
            let result = if let Some(tool) = tools.get(&tc.name) {
                match tc.args.as_object() {
                    Some(obj) => tool.call(obj).await,
                    None => tool.call(&serde_json::Map::new()).await,
                }
            } else {
                Err(anyhow::anyhow!("Unknown tool: {}", tc.name))
            };

            let result_str = match &result {
                Ok(s) => s.clone(),
                Err(e) => format!("Error: {}", e),
            };

            // Check if result requires claw interaction
            if let Ok(s) = &result {
                if let Ok(val) = serde_json::from_str::<Value>(s) {
                    if val.get("requires_claw").and_then(|v| v.as_bool()).unwrap_or(false) {
                        // This result needs to go back to claw - return early
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
                    if val.get("requires_registration").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(tool_info) = val.get("tool") {
                            if json_output {
                                let event = serde_json::json!({
                                    "event": "tool_created",
                                    "tool": tool_info,
                                });
                                println!("{}", serde_json::to_string(&event)?);
                            }
                        }
                    }
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
