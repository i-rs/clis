use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;

use crate::lsp::LspSession;
use crate::mcp::McpManager;
use crate::pty::PtyManager;

struct RuntimeInner {
    total_input_tokens: u64,
    total_output_tokens: u64,
    total_cost_cents: u64,
    max_cost_cents: u64,
    session_token_budget: u64,
    last_api_call: Instant,
    debug_mode: bool,
    verbose_mode: bool,
}

impl RuntimeInner {
    fn new() -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost_cents: 0,
            max_cost_cents: 0,
            session_token_budget: 0,
            last_api_call: Instant::now(),
            debug_mode: false,
            verbose_mode: false,
        }
    }
}

static RUNTIME: LazyLock<Mutex<Option<RuntimeInner>>> = LazyLock::new(|| Mutex::new(Some(RuntimeInner::new())));

fn with_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&mut RuntimeInner) -> R,
{
    let mut guard = RUNTIME.lock().unwrap_or_else(|e| e.into_inner());
    let inner = guard.as_mut().expect("Runtime not initialized");
    f(inner)
}

pub fn reset_for_testing() {
    *RUNTIME.lock().unwrap_or_else(|e| e.into_inner()) = Some(RuntimeInner::new());
    LSP_INIT.store(false, Ordering::Relaxed);
}

static LSP_INIT: AtomicBool = AtomicBool::new(false);

pub fn mark_lsp_initialized() {
    LSP_INIT.store(true, Ordering::Relaxed);
}

pub fn is_lsp_initialized() -> bool {
    LSP_INIT.load(Ordering::Relaxed)
}

pub fn mcp_manager() -> &'static McpManager {
    static MCP: LazyLock<McpManager> = LazyLock::new(McpManager::new);
    &MCP
}

pub fn pty_manager() -> &'static PtyManager {
    static PTY: LazyLock<PtyManager> = LazyLock::new(PtyManager::new);
    &PTY
}

pub fn lsp_session() -> &'static AsyncMutex<LspSession> {
    static LSP: LazyLock<AsyncMutex<LspSession>> = LazyLock::new(|| AsyncMutex::new(LspSession::new()));
    &LSP
}

pub fn last_web_request() -> &'static AsyncMutex<Instant> {
    static WEB: LazyLock<AsyncMutex<Instant>> = LazyLock::new(|| AsyncMutex::new(Instant::now()));
    &WEB
}

pub fn set_debug(enabled: bool) {
    with_runtime(|r| r.debug_mode = enabled);
}

pub fn is_debug() -> bool {
    with_runtime(|r| r.debug_mode)
}

pub fn set_verbose(enabled: bool) {
    with_runtime(|r| r.verbose_mode = enabled);
}

pub fn is_verbose() -> bool {
    with_runtime(|r| r.verbose_mode)
}

pub fn session_token_budget() -> u64 {
    with_runtime(|r| r.session_token_budget)
}

pub fn set_session_token_budget(budget: u64) {
    with_runtime(|r| r.session_token_budget = budget);
}

pub fn add_usage(input: u32, output: u32) {
    with_runtime(|r| {
        r.total_input_tokens += input as u64;
        r.total_output_tokens += output as u64;
    });
}

pub fn add_usage_with_cost(model: &str, input: u32, output: u32) {
    let (in_per_1k, out_per_1k) = model_pricing(model);
    let cost = (input as f64 / 1000.0) * in_per_1k + (output as f64 / 1000.0) * out_per_1k;
    with_runtime(|r| {
        r.total_input_tokens += input as u64;
        r.total_output_tokens += output as u64;
        r.total_cost_cents += (cost * 100.0) as u64;
    });
}

pub fn total_cost_cents() -> u64 {
    with_runtime(|r| r.total_cost_cents)
}

pub fn set_max_cost_dollars(dollars: f64) {
    with_runtime(|r| r.max_cost_cents = (dollars * 100.0) as u64);
}

pub fn exceeds_cost_budget() -> bool {
    with_runtime(|r| {
        r.max_cost_cents > 0 && r.total_cost_cents >= r.max_cost_cents
    })
}

fn model_pricing(model: &str) -> (f64, f64) {
    match model {
        "gpt-4o" => (2.5, 10.0),
        "gpt-4o-mini" => (0.15, 0.6),
        "gpt-4-turbo" => (10.0, 30.0),
        "gpt-4" => (30.0, 60.0),
        "gpt-3.5-turbo" => (0.5, 1.5),
        "o1" | "o1-mini" | "o3" | "o3-mini" => (15.0, 60.0),
        "claude-sonnet-4-20250514" | "claude-sonnet-4-5-20250514" => (3.0, 15.0),
        "claude-3-5-sonnet-20241022" => (3.0, 15.0),
        "claude-3-opus-20240229" => (15.0, 75.0),
        "claude-3-haiku-20240307" => (0.25, 1.25),
        _ => (1.0, 3.0),
    }
}

pub fn total_usage_tokens() -> u64 {
    with_runtime(|r| r.total_input_tokens + r.total_output_tokens)
}

pub fn exceeds_token_budget() -> bool {
    with_runtime(|r| {
        if r.session_token_budget == 0 {
            return false;
        }
        r.total_input_tokens + r.total_output_tokens >= r.session_token_budget
    })
}

pub fn reset_usage() {
    with_runtime(|r| {
        r.total_input_tokens = 0;
        r.total_output_tokens = 0;
    });
}

const MIN_REQUEST_INTERVAL_MS: u64 = 1000;

pub async fn rate_limit_wait() {
    let interval = Duration::from_millis(MIN_REQUEST_INTERVAL_MS);
    loop {
        let now = Instant::now();
        let last = with_runtime(|r| r.last_api_call);
        let elapsed = now.saturating_duration_since(last);
        if elapsed >= interval {
            with_runtime(|r| r.last_api_call = now);
            return;
        }
        tokio::time::sleep(interval - elapsed).await;
    }
}
