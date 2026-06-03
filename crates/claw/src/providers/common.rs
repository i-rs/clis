use crate::llm::LlmEvent;
use crate::stats::TokenRecord;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) fn truncate_body(body: &str, max_chars: usize) -> String {
    body.chars().take(max_chars).collect()
}

pub(crate) fn emit_usage_record(
    tx: &UnboundedSender<LlmEvent>,
    model: &str,
    provider: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
    has_tool_calls: bool,
    tool_call_count: u32,
    latency_ms: u64,
) {
    let _ = tx.send(LlmEvent::UsageRecord(TokenRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        agent_id: "default".to_string(),
        model: model.to_string(),
        provider: provider.to_string(),
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
        has_tool_calls,
        tool_call_count,
        react_rounds: 0,
        success: true,
        latency_ms,
        estimated_cost_usd: 0.0,
        trace_id: String::new(),
    }));
}

pub(crate) fn emit_http_log(
    tx: &UnboundedSender<LlmEvent>,
    status: u16,
    duration_ms: u64,
    model: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
    error: Option<String>,
    request_body: &str,
) {
    let _ = tx.send(LlmEvent::HttpLog(crate::llm::HttpLogData {
        status,
        duration_ms,
        model: model.to_string(),
        prompt_tokens,
        completion_tokens,
        error,
        request_body: truncate_body(request_body, 2000),
    }));
}
