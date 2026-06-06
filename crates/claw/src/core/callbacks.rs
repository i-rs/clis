use i_rs_claw_core::error::ErrorCategory;
use i_rs_claw_core::llm::TokenUsage;
use serde_json::Value;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolExecutionRecord {
    pub name: String,
    pub args: Value,
    pub result: String,
    pub category: ErrorCategory,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LlmCallRecord {
    pub model: String,
    pub provider: String,
    pub message_count: usize,
    pub tool_count: usize,
    pub usage: Option<TokenUsage>,
    pub elapsed_ms: u64,
}

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait AgentCallbacks: Send + Sync {
    fn on_tool_start(&self, _name: &str, _args: &Value) {}
    fn on_tool_end(&self, _record: &ToolExecutionRecord) {}
    fn on_llm_start(&self, _messages: &[Value], _tool_schemas: &[Value]) {}
    fn on_llm_end(&self, _record: &LlmCallRecord) {}
    fn on_error(&self, _error: &str, _category: ErrorCategory) {}
    fn on_round_start(&self, _round: u32, _max_rounds: u32) {}
    fn on_round_end(&self, _round: u32, _had_tool_calls: bool) {}
    fn on_session_start(&self, _session_id: &str) {}
    fn on_session_end(&self, _session_id: &str, _total_rounds: u32) {}
}

pub struct CallbackChain {
    callbacks: Vec<Box<dyn AgentCallbacks>>,
}

impl CallbackChain {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add(mut self, cb: Box<dyn AgentCallbacks>) -> Self {
        self.callbacks.push(cb);
        self
    }

    pub fn with(mut self, cb: Box<dyn AgentCallbacks>) -> Self {
        self.callbacks.push(cb);
        self
    }
}

impl Default for CallbackChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AgentCallbacks for CallbackChain {
    fn on_tool_start(&self, name: &str, args: &Value) {
        for cb in &self.callbacks {
            cb.on_tool_start(name, args);
        }
    }
    fn on_tool_end(&self, record: &ToolExecutionRecord) {
        for cb in &self.callbacks {
            cb.on_tool_end(record);
        }
    }
    fn on_llm_start(&self, messages: &[Value], tool_schemas: &[Value]) {
        for cb in &self.callbacks {
            cb.on_llm_start(messages, tool_schemas);
        }
    }
    fn on_llm_end(&self, record: &LlmCallRecord) {
        for cb in &self.callbacks {
            cb.on_llm_end(record);
        }
    }
    fn on_error(&self, error: &str, category: ErrorCategory) {
        for cb in &self.callbacks {
            cb.on_error(error, category);
        }
    }
    fn on_round_start(&self, round: u32, max_rounds: u32) {
        for cb in &self.callbacks {
            cb.on_round_start(round, max_rounds);
        }
    }
    fn on_round_end(&self, round: u32, had_tool_calls: bool) {
        for cb in &self.callbacks {
            cb.on_round_end(round, had_tool_calls);
        }
    }
    fn on_session_start(&self, session_id: &str) {
        for cb in &self.callbacks {
            cb.on_session_start(session_id);
        }
    }
    fn on_session_end(&self, session_id: &str, total_rounds: u32) {
        for cb in &self.callbacks {
            cb.on_session_end(session_id, total_rounds);
        }
    }
}

pub struct LoggingCallback;

impl LoggingCallback {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoggingCallback {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AgentCallbacks for LoggingCallback {
    fn on_tool_start(&self, name: &str, args: &Value) {
        tracing::info!(tool = %name, args = %args, "callback: tool_start");
    }
    fn on_tool_end(&self, record: &ToolExecutionRecord) {
        tracing::info!(
            tool = %record.name,
            category = ?record.category,
            elapsed_ms = %record.elapsed_ms,
            result_len = %record.result.len(),
            "callback: tool_end"
        );
    }
    fn on_llm_end(&self, record: &LlmCallRecord) {
        tracing::info!(
            model = %record.model,
            provider = %record.provider,
            elapsed_ms = %record.elapsed_ms,
            usage = ?record.usage,
            "callback: llm_end"
        );
    }
    fn on_error(&self, error: &str, category: ErrorCategory) {
        tracing::warn!(error = %error, category = ?category, "callback: error");
    }
    fn on_session_end(&self, session_id: &str, total_rounds: u32) {
        tracing::info!(session_id = %session_id, total_rounds = %total_rounds, "callback: session_end");
    }
}

pub struct AuditLogCallback {
    records: std::sync::Mutex<Vec<AuditRecord>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuditRecord {
    pub timestamp: i64,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub detail: String,
}

impl AuditLogCallback {
    pub fn new() -> Self {
        Self {
            records: std::sync::Mutex::new(Vec::new()),
        }
    }

    #[allow(dead_code)]
    pub fn drain(&self) -> Vec<AuditRecord> {
        let mut guard = self.records.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut *guard)
    }

    #[allow(dead_code)]
    fn push(&self, event_type: &str, tool_name: Option<&str>, detail: String) {
        let mut guard = self.records.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(AuditRecord {
            timestamp: chrono::Utc::now().timestamp(),
            event_type: event_type.to_string(),
            tool_name: tool_name.map(|s| s.to_string()),
            detail,
        });
    }
}

impl Default for AuditLogCallback {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AgentCallbacks for AuditLogCallback {
    fn on_tool_start(&self, name: &str, _args: &Value) {
        self.push("tool_start", Some(name), format!("工具开始执行: {}", name));
    }
    fn on_tool_end(&self, record: &ToolExecutionRecord) {
        self.push(
            "tool_end",
            Some(&record.name),
            format!(
                "工具完成: {} ({:?}, {}ms)",
                record.name, record.category, record.elapsed_ms
            ),
        );
    }
    fn on_error(&self, error: &str, category: ErrorCategory) {
        self.push("error", None, format!("[{:?}] {}", category, error));
    }
    fn on_session_start(&self, session_id: &str) {
        self.push("session_start", None, format!("会话开始: {}", session_id));
    }
    fn on_session_end(&self, session_id: &str, total_rounds: u32) {
        self.push(
            "session_end",
            None,
            format!("会话结束: {} ({}轮)", session_id, total_rounds),
        );
    }
}
