use crate::config::Config;
use crate::utils;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    #[allow(dead_code)]
    pub total_tokens: u32,
}

#[derive(Debug)]
pub enum LlmEvent {
    /// A text token from the streaming response
    Token(String),
    /// Signals the app to start a new assistant message (for multi-round responses)
    NewRound,
    /// A tool was executed (with result)
    ToolExecuted {
        name: String,
        args: String,
        result: String,
    },
    /// Real-time status update (shown in status bar)
    Status(String),
    /// An error occurred
    Error(String),
    /// All responses complete, carries final API message list and optional token usage
    Done(Vec<Value>, Option<TokenUsage>),
}

#[derive(Default, Clone)]
struct ToolCallAcc {
    id: String,
    name: String,
    arguments: String,
}

enum StreamResult {
    Text(Option<TokenUsage>),
    ToolCalls(Vec<(ToolCallAcc, Value)>, String), // tool_calls + accumulated reasoning_content
}

/// Stream chat completion and parse SSE events
async fn stream_chat(
    client: &reqwest::Client,
    messages: &[Value],
    config: &Config,
    tool_schemas: &[Value],
    tx: &mpsc::UnboundedSender<LlmEvent>,
) -> anyhow::Result<StreamResult> {
    let mut body = serde_json::json!({
        "model": config.model,
        "messages": messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });

    if !tool_schemas.is_empty() {
        body["tools"] = Value::Array(tool_schemas.to_vec());
        body["parallel_tool_calls"] = serde_json::Value::Bool(false);
    }

    let response = client
        .post(format!("{}/chat/completions", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("HTTP-Referer", "https://github.com/i-rs/clis")
        .header("X-Title", "i-rs-claw")
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("API 请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("API 返回错误 {}: {}", status, text));
    }

    let mut stream = response.bytes_stream();
    let mut buf = String::new();
    let mut tool_calls: Vec<ToolCallAcc> = Vec::new();
    let mut finish_reason = String::new();
    let mut reasoning_buf = String::new();
    let mut usage: Option<TokenUsage> = None;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| anyhow::anyhow!("流读取失败: {}", e))?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete SSE lines
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            buf = buf[pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    continue;
                }

                if let Ok(parsed) = serde_json::from_str::<Value>(data.trim()) {
                    // Check for usage data (final chunk with stream_options.include_usage)
                    if let Some(usage_data) = parsed.get("usage") {
                        if !usage_data.is_null() {
                            usage = Some(TokenUsage {
                                prompt_tokens: usage_data["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                                completion_tokens: usage_data["completion_tokens"].as_u64().unwrap_or(0) as u32,
                                total_tokens: usage_data["total_tokens"].as_u64().unwrap_or(0) as u32,
                            });
                        }
                    }

                    if let Some(choices) = parsed["choices"].as_array() {
                        if let Some(choice) = choices.first() {
                            if let Some(reason) = choice["finish_reason"].as_str() {
                                if !reason.is_empty() && reason != "null" && reason != "stop" {
                                    finish_reason = reason.to_string();
                                }
                            }

                            if let Some(delta) = choice.get("delta") {
                                // Accumulate reasoning_content (DeepSeek requires echoing it back)
                                if let Some(rc) = delta.get("reasoning_content").and_then(|r| r.as_str()) {
                                    reasoning_buf.push_str(rc);
                                }

                                // Text content
                                if let Some(text) = delta.get("content").and_then(|c| c.as_str()) {
                                    if !text.is_empty() {
                                        let _ = tx.send(LlmEvent::Token(text.to_string()));
                                    }
                                }

                                // Tool calls (streaming delta)
                                if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array())
                                {
                                    for tc in tcs {
                                        let idx = tc
                                            .get("index")
                                            .and_then(|i| i.as_i64())
                                            .unwrap_or(0)
                                            as usize;
                                        if idx >= tool_calls.len() {
                                            tool_calls.resize(idx + 1, ToolCallAcc::default());
                                        }
                                        if let Some(id) = tc.get("id").and_then(|i| i.as_str()) {
                                            tool_calls[idx].id = id.to_string();
                                        }
                                        if let Some(func) = tc.get("function") {
                                            if let Some(name) =
                                                func.get("name").and_then(|n| n.as_str())
                                            {
                                                tool_calls[idx].name = name.to_string();
                                            }
                                            if let Some(args) =
                                                func.get("arguments").and_then(|a| a.as_str())
                                            {
                                                tool_calls[idx].arguments.push_str(args);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if finish_reason == "tool_calls" && !tool_calls.is_empty()
        && tool_calls.iter().any(|tc| !tc.id.is_empty())
    {
        let mut parsed = Vec::new();
        for tc in &tool_calls {
            let args: Value =
                serde_json::from_str(&tc.arguments).unwrap_or(serde_json::json!({}));
            parsed.push((
                ToolCallAcc {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    arguments: tc.arguments.clone(),
                },
                args,
            ));
        }
        return Ok(StreamResult::ToolCalls(parsed, reasoning_buf));
    }

    Ok(StreamResult::Text(usage))
}

/// Load system prompt from external file and inject dynamic layers.
///
/// Layers:
/// 1. Static behavior prompt (system.md)
/// 2. Tool index (from TOOL_INDEX static data)
/// 3. Hot tool docs (skill teach outputs for frequently used tools)
/// 4. User memory (cross-session preferences and history)
fn build_system_prompt(
    tool_index: &str,
    hot_tools: &str,
    user_memory: &str,
) -> String {
    let mut prompt = include_str!("../prompts/system.md").to_string();
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let weekday = now.format("%A").to_string();
    prompt = prompt
        .replace("{current_date}", &today)
        .replace("{current_weekday}", &weekday);

    prompt = prompt.replace("{{TOOL_INDEX}}", tool_index);
    prompt = prompt.replace("{{HOT_TOOLS}}", hot_tools);
    prompt = prompt.replace("{{USER_MEMORY}}", user_memory);

    prompt
}

/// Convert app messages to API-compatible message list.
/// If `saved_api_messages` exists, reuse them as base (preserving tool call context)
/// and only append the new user message.
pub fn build_messages(
    app_messages: &[crate::app::Message],
    user_text: &str,
    saved_api_messages: &Option<Vec<Value>>,
    tool_index: &str,
    hot_tools: &str,
    user_memory: &str,
) -> Vec<Value> {
    if let Some(prev_msgs) = saved_api_messages {
        // Reuse saved API messages (has full context including tool calls)
        let mut msgs = prev_msgs.clone();
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

        // Keep last ~20 messages for context window
        if msgs.len() > 21 {
            let system = msgs[0].clone();
            msgs = msgs.split_off(msgs.len() - 20);
            msgs.insert(0, system);
        }
        return msgs;
    }

    // First turn: build from scratch
    let mut msgs = vec![serde_json::json!({
        "role": "system",
        "content": build_system_prompt(tool_index, hot_tools, user_memory)
    })];

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
fn execute_tool_call(
    name: &str,
    args: &Value,
) -> String {
    match name {
        "search_tools" => {
            let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
            crate::tools::search::search(query)
        }
        "i_rs" => {
            let tool = args.get("tool").and_then(|t| t.as_str()).unwrap_or("");
            let cmd = args.get("command").and_then(|c| c.as_str()).unwrap_or("");
            let cmd_args: Vec<String> = args
                .get("args")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            match crate::tools::i_rs_cmd::execute(tool, cmd, &cmd_args) {
                Ok(r) => r,
                Err(e) => format!("错误: {}", e),
            }
        }
        _ => format!("未知工具: {}", name),
    }
}

/// Main chat loop: stream, handle tool calls, continue until done
pub async fn chat_loop(
    config: Config,
    messages: Vec<Value>,
    tx: mpsc::UnboundedSender<LlmEvent>,
) {
    let enabled = if config.enabled_tools.is_empty() {
        None
    } else {
        Some(&config.enabled_tools)
    };
    let tool_schemas = crate::tools::get_tool_schemas(enabled);
    let mut msgs = messages;

    // Reuse HTTP client across retries
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(LlmEvent::Error(format!("创建 HTTP 客户端失败: {}", e)));
            return;
        }
    };

    loop {
        let _ = tx.send(LlmEvent::NewRound);
        let _ = tx.send(LlmEvent::Status("🤔 思考中…".to_string()));

        match stream_chat(&client, &msgs, &config, &tool_schemas, &tx).await {
            Ok(StreamResult::Text(usage)) => {
                let _ = tx.send(LlmEvent::Done(msgs.clone(), usage));
                break;
            }
            Ok(StreamResult::ToolCalls(calls, reasoning_content)) => {
                // Add assistant message with tool_calls to history
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

                // Execute each tool call
                for (tc, args) in &calls {
                    let status_msg = if tc.name == "i_rs" {
                        let tool = args.get("tool").and_then(|t| t.as_str()).unwrap_or("");
                        let cmd = args.get("command").and_then(|c| c.as_str()).unwrap_or("");
                        format!("⚡ 调用工具: {} {}", tool, cmd)
                    } else {
                        format!("⚡ 调用工具: {}", tc.name)
                    };
                    let _ = tx.send(LlmEvent::Status(status_msg));
                    let result = execute_tool_call(&tc.name, args);

                    let _ = tx.send(LlmEvent::ToolExecuted {
                        name: tc.name.clone(),
                        args: args.to_string(),
                        result: if result.len() > 200 {
                            format!("{}...(truncated)", utils::truncate(&result, 200))
                        } else {
                            result.clone()
                        },
                    });

                    // Add tool result to conversation history
                    let trimmed = if result.len() > 500 {
                        format!("{}...(truncated)", utils::truncate(&result, 500))
                    } else {
                        result.clone()
                    };
                    msgs.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tc.id,
                        "content": trimmed,
                    }));
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
