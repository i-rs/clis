use crate::llm::{LlmEvent, StreamResult, TokenUsage, ToolCallAcc};
use crate::stats::TokenRecord;
use futures_util::StreamExt;
use serde_json::Value;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

/// Minimum backoff base for 429 rate-limit retries (ms).
const RETRY_429_BASE_MS: u64 = 1000;
/// Minimum backoff base for 5xx server error retries (ms).
const RETRY_5XX_BASE_MS: u64 = 2000;
/// Minimum backoff base for network error retries (ms).
const RETRY_NETWORK_BASE_MS: u64 = 1000;
/// Maximum jitter added to backoff (ms).
const RETRY_JITTER_MS: u64 = 1000;

/// Send an HTTP POST request with exponential backoff retry.
///
/// Retry policy:
/// - 429 Too Many Requests: parse `Retry-After` header, fallback to exponential backoff + jitter
/// - 5xx Server Error: exponential backoff + jitter
/// - Network errors: exponential backoff + jitter
/// - 4xx Client Errors (auth, bad request, etc.): **no retry**, fail immediately
/// - Max `max_attempts` attempts
///
/// Returns `Ok(response)` on success or when retries exhausted on retriable errors.
/// Returns `Err(...)` when retries exhausted on network errors or on non-retriable HTTP errors.
#[tracing::instrument(skip(client, body, headers))]
pub(crate) async fn send_with_retry(
    max_attempts: u32,
    client: &reqwest::Client,
    url: &str,
    body: &serde_json::Value,
    headers: &[(String, String)],
) -> anyhow::Result<reqwest::Response> {
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        let mut req = client.post(url).json(body);
        for (key, value) in headers {
            req = req.header(key.as_str(), value.as_str());
        }
        match req.send().await {
            Ok(r) if r.status().as_u16() == 429 => {
                if attempt >= max_attempts {
                    let text = r.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "API 限流 (429) 重试{}次后仍失败: {}",
                        attempt,
                        text
                    ));
                }
                let wait_ms = parse_retry_after_ms(&r).unwrap_or_else(|| {
                    (RETRY_429_BASE_MS << attempt.min(4).saturating_sub(1).min(60))
                        + fastrand::u64(0..RETRY_JITTER_MS / 2)
                });
                tracing::warn!(
                    "API 限流 (429), 等待 {}ms 后重试 ({}/{})",
                    wait_ms,
                    attempt,
                    max_attempts
                );
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                continue;
            }
            Ok(r) if r.status().is_server_error() => {
                if attempt >= max_attempts {
                    let status = r.status().as_u16();
                    let text = r.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "API 服务器错误 ({}) 重试{}次后仍失败: {}",
                        status,
                        attempt,
                        text
                    ));
                }
                let wait_ms = (RETRY_5XX_BASE_MS << attempt.min(3).saturating_sub(1).min(60))
                    + fastrand::u64(0..RETRY_JITTER_MS);
                tracing::warn!(
                    "API 服务器错误 ({}), 等待 {}ms 后重试 ({}/{})",
                    r.status(),
                    wait_ms,
                    attempt,
                    max_attempts
                );
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                continue;
            }
            Ok(r) if r.status().is_client_error() && r.status().as_u16() != 429 => {
                let status = r.status().as_u16();
                let text = r.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("API 客户端错误 ({}): {}", status, text));
            }
            Ok(r) => return Ok(r),
            Err(e) => {
                if attempt >= max_attempts {
                    return Err(anyhow::anyhow!("API 请求失败 (重试{}次): {}", attempt, e));
                }
                let wait_ms = (RETRY_NETWORK_BASE_MS << attempt.min(3).saturating_sub(1).min(60))
                    + fastrand::u64(0..RETRY_JITTER_MS / 2);
                tracing::warn!(
                    "API 网络错误, 等待 {}ms 后重试 ({}/{}): {}",
                    wait_ms,
                    attempt,
                    max_attempts,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                continue;
            }
        }
    }
}

/// Parse Retry-After header from response (seconds or HTTP-date).
/// Returns milliseconds if header is present and parseable.
fn parse_retry_after_ms(response: &reqwest::Response) -> Option<u64> {
    let header_val = response.headers().get("retry-after")?.to_str().ok()?;
    if let Ok(secs) = header_val.parse::<u64>() {
        Some(secs * 1000)
    } else {
        None
    }
}

/// Internal streaming logic shared by OpenAI-compatible providers
/// (OpenAI, Ollama, and any other OpenAI-format endpoints).
#[tracing::instrument(skip(client, tx))]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn openai_stream_chat_impl(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    model: &str,
    provider_kind: &str,
    messages: &[Value],
    tool_schemas: &[Value],
    tx: &UnboundedSender<LlmEvent>,
) -> anyhow::Result<StreamResult> {
    let start = Instant::now();
    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });

    if !tool_schemas.is_empty() {
        body["tools"] = Value::Array(tool_schemas.to_vec());
        body["parallel_tool_calls"] = serde_json::Value::Bool(true);
    }

    let body_json = serde_json::to_string(&body).unwrap_or_default();

    let headers: Vec<(String, String)> = if let Some(key) = api_key {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", key)),
            (
                "HTTP-Referer".to_string(),
                "https://github.com/i-rs/clis".to_string(),
            ),
            ("X-Title".to_string(), "i-rs-claw".to_string()),
        ]
    } else {
        Vec::new()
    };
    let response = send_with_retry(3, client, url, &body, &headers).await?;

    let status = response.status().as_u16();

    if response.status().is_success() {
        let mut stream = response.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        let mut tool_calls: Vec<ToolCallAcc> = Vec::new();
        let mut reasoning_buf = String::new();
        let mut content_buf = String::new();
        let mut usage: Option<TokenUsage> = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("流读取失败: {}", e))?;
            buf.extend_from_slice(&chunk);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line_bytes).trim().to_string();

                if line.is_empty() {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        continue;
                    }

                    if let Ok(parsed) = serde_json::from_str::<Value>(data.trim()) {
                        // Usage data (final chunk with include_usage)
                        if let Some(usage_data) = parsed.get("usage")
                            && !usage_data.is_null()
                        {
                            usage = Some(TokenUsage {
                                prompt_tokens: usage_data["prompt_tokens"].as_u64().unwrap_or(0)
                                    as u32,
                                completion_tokens: usage_data["completion_tokens"]
                                    .as_u64()
                                    .unwrap_or(0)
                                    as u32,
                                total_tokens: usage_data["total_tokens"].as_u64().unwrap_or(0)
                                    as u32,
                            });
                        }

                        if let Some(choices) = parsed["choices"].as_array()
                            && let Some(choice) = choices.first()
                            && let Some(delta) = choice.get("delta")
                        {
                            // Accumulate reasoning_content (DeepSeek)
                            if let Some(rc) =
                                delta.get("reasoning_content").and_then(|r| r.as_str())
                            {
                                reasoning_buf.push_str(rc);
                                let _ = tx.send(LlmEvent::Reasoning(rc.to_string()));
                            }

                            // Text content
                            if let Some(text) = delta.get("content").and_then(|c| c.as_str())
                                && !text.is_empty()
                            {
                                content_buf.push_str(text);
                                let _ = tx.send(LlmEvent::Token(text.to_string()));
                            }

                            // Tool calls (streaming delta)
                            if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                for tc in tcs {
                                    let idx = tc.get("index").and_then(|i| i.as_i64()).unwrap_or(0)
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

        let duration_ms = start.elapsed().as_millis() as u64;
        let prompt_tokens = usage.map(|u| u.prompt_tokens).unwrap_or(0);
        let completion_tokens = usage.map(|u| u.completion_tokens).unwrap_or(0);
        let has_tool_calls =
            !tool_calls.is_empty() && tool_calls.iter().any(|tc| !tc.id.is_empty());
        let tool_call_count = if has_tool_calls {
            tool_calls.len() as u32
        } else {
            0
        };

        // Emit usage record for statistics
        let _ = tx.send(LlmEvent::UsageRecord(TokenRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            agent_id: "default".to_string(),
            model: model.to_string(),
            provider: provider_kind.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            has_tool_calls,
            tool_call_count,
            react_rounds: 0, // Will be updated by chat_loop if needed
            success: true,
            latency_ms: duration_ms,
            estimated_cost_usd: 0.0,
            trace_id: String::new(),
        }));

        let _ = tx.send(LlmEvent::HttpLog(crate::llm::HttpLogData {
            status,
            duration_ms,
            model: model.to_string(),
            prompt_tokens,
            completion_tokens,
            error: None,
            request_body: body_json.chars().take(2000).collect::<String>(),
        }));

        if has_tool_calls {
            let mut parsed = Vec::new();
            for tc in &tool_calls {
                let args: Value = serde_json::from_str(&tc.arguments).unwrap_or_else(|e| {
                    tracing::warn!("工具 '{}' 参数 JSON 解析失败: {}", tc.name, e);
                    serde_json::json!({})
                });
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

        Ok(StreamResult::Text(usage, content_buf, reasoning_buf))
    } else {
        let text = response.text().await.unwrap_or_default();
        let duration_ms = start.elapsed().as_millis() as u64;
        let _ = tx.send(LlmEvent::HttpLog(crate::llm::HttpLogData {
            status,
            duration_ms,
            model: model.to_string(),
            prompt_tokens: 0,
            completion_tokens: 0,
            error: Some(format!("HTTP {}: {}", status, text)),
            request_body: body_json.chars().take(2000).collect::<String>(),
        }));
        Err(anyhow::anyhow!("API 返回错误 {}: {}", status, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_retry_after_seconds() {
        let resp = test_response_with_header("retry-after", "5");
        assert_eq!(parse_retry_after_ms(&resp), Some(5000));
    }

    #[test]
    fn test_parse_retry_after_zero() {
        let resp = test_response_with_header("retry-after", "0");
        let result = parse_retry_after_ms(&resp);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_parse_retry_after_missing() {
        let resp = reqwest::Response::from(http::Response::builder().status(200).body("").unwrap());
        assert_eq!(parse_retry_after_ms(&resp), None);
    }

    fn test_response_with_header(key: &str, value: &str) -> reqwest::Response {
        reqwest::Response::from(
            http::Response::builder()
                .status(200)
                .header(key, value)
                .body("")
                .unwrap(),
        )
    }
}
