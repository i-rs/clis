use crate::llm::LlmEvent;
use crate::stats::TokenRecord;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) fn truncate_body(body: &str, max_chars: usize) -> String {
    body.chars().take(max_chars).collect()
}

/// Dump the full request body to a file when `CLAW_DUMP_PROMPTS` env var is set.
/// Path format: `{CLAW_DUMP_PROMPTS}/prompt-{timestamp}.json`
pub(crate) fn dump_prompt_body(body: &serde_json::Value) {
    let Ok(dir) = std::env::var("CLAW_DUMP_PROMPTS") else {
        return;
    };
    let dump_dir = std::path::PathBuf::from(&dir);
    let _ = std::fs::create_dir_all(&dump_dir);
    let path = dump_dir.join(format!(
        "prompt-{}.json",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    ));
    let Ok(pretty) = serde_json::to_string_pretty(body) else {
        return;
    };
    if std::fs::write(&path, &pretty).is_ok() {
        tracing::info!("prompt 已导出到 {}", path.display());
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_usage_record(
    tx: &UnboundedSender<LlmEvent>,
    model: &str,
    provider: &str,
    prompt_tokens: u32,
    completion_tokens: u32,
    has_tool_calls: bool,
    tool_call_count: u32,
    latency_ms: u64,
    trace_id: &str,
) {
    let _ = tx.send(LlmEvent::UsageRecord(TokenRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        user_id: "default".to_string(),
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
        trace_id: trace_id.to_string(),
    }));
}

#[allow(clippy::too_many_arguments)]
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
