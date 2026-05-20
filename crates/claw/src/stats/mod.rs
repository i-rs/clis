pub mod aggregator;
pub mod pricing;
pub(crate) mod store;

use pricing::ModelPricingTable;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

// ── Config ──

/// Stats configuration section in config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig {
    /// Whether to enable token usage statistics (default: true).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Custom model pricing overrides.
    #[serde(default)]
    pub pricing: std::collections::HashMap<String, StatsPricingConfig>,
    /// Number of days to retain stats (0 = keep forever).
    #[serde(default = "default_keep_days")]
    pub keep_days: u32,
}

fn default_enabled() -> bool { true }
fn default_keep_days() -> u32 { 90 }

impl Default for StatsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pricing: std::collections::HashMap::new(),
            keep_days: 90,
        }
    }
}

/// Per-model pricing override in config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsPricingConfig {
    pub input: f64,
    pub output: f64,
}

// ── Data Models ──

/// Single LLM request token usage record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecord {
    /// Unique ID (UUID v4)
    pub id: String,
    /// Request timestamp (Unix epoch seconds)
    pub timestamp: i64,
    /// Agent ID
    pub agent_id: String,
    /// Model name (e.g. "gpt-4o-mini", "claude-sonnet-4-20250514")
    pub model: String,
    /// Provider type (e.g. "openai", "anthropic")
    pub provider: String,
    /// Input token count
    pub prompt_tokens: u32,
    /// Output token count
    pub completion_tokens: u32,
    /// Total token count
    pub total_tokens: u32,
    /// Whether the response includes tool calls
    pub has_tool_calls: bool,
    /// Number of tool calls in this response
    pub tool_call_count: u32,
    /// ReAct round number (which LLM call in the chain this is)
    pub react_rounds: u32,
    /// Whether the request succeeded
    pub success: bool,
    /// Request latency in milliseconds
    pub latency_ms: u64,
    /// Estimated cost in USD
    pub estimated_cost_usd: f64,
}

// ── Aggregation Results ──

/// Aggregated token usage statistics for display.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct TokenStats {
    pub period: StatsPeriod,
    pub total_requests: u32,
    pub total_tokens: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_cost_usd: f64,
    pub avg_tokens_per_request: f64,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub by_model: Vec<ModelStats>,
    pub by_agent: Vec<AgentStats>,
    pub daily_series: Vec<DailyStats>,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub enum StatsPeriod {
    Today,
    Last7Days,
    Last30Days,
    All,
    Custom { from: i64, to: i64 },
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelStats {
    pub model: String,
    pub request_count: u32,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStats {
    pub agent_id: String,
    pub request_count: u32,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyStats {
    pub date: String,
    pub request_count: u32,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
}

/// Lightweight today's summary for status bar display.
#[derive(Debug, Clone, Default)]
pub struct TodaySummary {
    pub requests: u32,
    pub tokens: u64,
    pub cost_usd: f64,
}

// ── StatsManager ──

/// Manages token usage statistics collection, persistence, and querying.
///
/// Records are buffered in memory and flushed to a JSONL file periodically.
/// No tokio dependency — uses a simple `Mutex<Vec>` for thread-safe buffering.
pub struct StatsManager {
    /// Path to the usage JSONL file.
    store_path: PathBuf,
    /// Pricing table for cost estimation.
    #[allow(dead_code)]
    pricing: ModelPricingTable,
    /// In-memory buffer of unsaved records.
    buffer: Mutex<Vec<TokenRecord>>,
    /// Max records to buffer before auto-flush.
    flush_threshold: usize,
}

impl StatsManager {
    /// Create a new StatsManager.
    ///
    /// `claw_dir` is the base directory for claw data (~/.i-rs-claw/claw).
    /// Stats are stored at `{claw_dir}/stats/usage.jsonl`.
    pub fn new(claw_dir: &std::path::Path, config: &StatsConfig) -> Self {
        let store_path = claw_dir.join("stats").join("usage.jsonl");
        let mut pricing = ModelPricingTable::new();

        if !config.pricing.is_empty() {
            pricing.apply_overrides(&config.pricing);
        }

        Self {
            store_path,
            pricing,
            buffer: Mutex::new(Vec::with_capacity(50)),
            flush_threshold: 50,
        }
    }

    /// Record a single token usage event.
    ///
    /// This is O(1) and does not block — just pushes to the in-memory buffer.
    /// The buffer is flushed to disk when it reaches `flush_threshold` or
    /// when [`flush`](Self::flush) is explicitly called.
    pub fn record(&self, record: TokenRecord) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.push(record);

        if buffer.len() >= self.flush_threshold {
            // Flush synchronously — fast since JSONL append is O(1) per record
            let records = std::mem::take(&mut *buffer);
            if let Err(e) = store::append_records(&self.store_path, &records) {
                tracing::error!("刷写 token 统计失败: {}", e);
            }
        }
    }

    /// Flush all buffered records to disk.
    pub fn flush(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            return;
        }
        let records = std::mem::take(&mut *buffer);
        if let Err(e) = store::append_records(&self.store_path, &records) {
            tracing::error!("刷写 token 统计失败: {}", e);
        }
    }

    /// Create a TokenRecord from an LLM call event.
    #[allow(dead_code)]
    pub fn create_record(
        &self,
        agent_id: &str,
        model: &str,
        provider: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        has_tool_calls: bool,
        tool_call_count: u32,
        react_rounds: u32,
        success: bool,
        latency_ms: u64,
    ) -> TokenRecord {
        let total_tokens = prompt_tokens + completion_tokens;
        let estimated_cost_usd = self.pricing.estimate(model, prompt_tokens, completion_tokens);

        TokenRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Local::now().timestamp(),
            agent_id: agent_id.to_string(),
            model: model.to_string(),
            provider: provider.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens,
            has_tool_calls,
            tool_call_count,
            react_rounds,
            success,
            latency_ms,
            estimated_cost_usd,
        }
    }

    /// Get today's summary from the store file + in-memory buffer.
    pub fn today_summary(&self) -> TodaySummary {
        let mut records = match store::read_range(&self.store_path, None, None) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("读取 token 统计失败: {}", e);
                Vec::new()
            }
        };

        // Include buffered unsaved records
        if let Ok(buffer) = self.buffer.lock() {
            records.extend(buffer.iter().cloned());
        }

        aggregator::today_summary(&records)
    }

    /// Query aggregated stats for a time period.
    #[allow(dead_code)]
    pub fn query(&self, period: StatsPeriod) -> TokenStats {
        let records = match store::read_range(&self.store_path, None, None) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("读取 token 统计失败: {}", e);
                Vec::new()
            }
        };

        let mut result = aggregator::aggregate(&records, &self.pricing);
        result.period = period;
        result
    }

    /// Get the estimated cost for a given model and token counts.
    #[allow(dead_code)]
    pub fn estimate_cost(&self, model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        self.pricing.estimate(model, prompt_tokens, completion_tokens)
    }

    /// Flush buffer then remove records older than `keep_days`.
    /// Call this periodically (e.g., on startup or exit).
    pub fn cleanup(&self, keep_days: u32) {
        self.flush();
        if keep_days == 0 {
            return;
        }
        if let Err(e) = store::prune_old_records(&self.store_path, keep_days) {
            tracing::error!("清理过期统计记录失败: {}", e);
        }
    }
}
