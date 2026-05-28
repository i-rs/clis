use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::mcp::McpManager;
use crate::pty::PtyManager;
use crate::lsp::LspSession;

pub static MCP_MANAGER: LazyLock<McpManager> = LazyLock::new(McpManager::new);
pub static PTY_MANAGER: LazyLock<PtyManager> = LazyLock::new(PtyManager::new);
pub static LSP_SESSION: LazyLock<Mutex<LspSession>> = LazyLock::new(|| Mutex::new(LspSession::new()));
pub static LSP_INITIALIZED: AtomicBool = AtomicBool::new(false);
pub static LAST_WEB_REQUEST: LazyLock<Mutex<Instant>> = LazyLock::new(|| Mutex::new(Instant::now()));

static TOTAL_INPUT_TOKENS: AtomicU64 = AtomicU64::new(0);
static TOTAL_OUTPUT_TOKENS: AtomicU64 = AtomicU64::new(0);
static SESSION_TOKEN_BUDGET: AtomicU64 = AtomicU64::new(0);
static LAST_API_CALL: LazyLock<std::sync::Mutex<Instant>> = LazyLock::new(|| std::sync::Mutex::new(Instant::now()));
static MIN_REQUEST_INTERVAL_MS: u64 = 1000;
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
static VERBOSE_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_debug(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
}

pub fn is_debug() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

pub fn set_verbose(enabled: bool) {
    VERBOSE_MODE.store(enabled, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE_MODE.load(Ordering::Relaxed)
}

pub fn session_token_budget() -> u64 {
    SESSION_TOKEN_BUDGET.load(Ordering::Relaxed)
}

pub fn set_session_token_budget(budget: u64) {
    SESSION_TOKEN_BUDGET.store(budget, Ordering::Relaxed);
}

pub fn add_usage(input: u32, output: u32) {
    TOTAL_INPUT_TOKENS.fetch_add(input as u64, Ordering::Relaxed);
    TOTAL_OUTPUT_TOKENS.fetch_add(output as u64, Ordering::Relaxed);
}

pub fn total_usage_tokens() -> u64 {
    TOTAL_INPUT_TOKENS.load(Ordering::Relaxed) + TOTAL_OUTPUT_TOKENS.load(Ordering::Relaxed)
}

pub fn exceeds_token_budget() -> bool {
    let budget = SESSION_TOKEN_BUDGET.load(Ordering::Relaxed);
    if budget == 0 { return false; }
    total_usage_tokens() >= budget
}

pub fn reset_usage() {
    TOTAL_INPUT_TOKENS.store(0, Ordering::Relaxed);
    TOTAL_OUTPUT_TOKENS.store(0, Ordering::Relaxed);
}

pub async fn rate_limit_wait() {
    let interval = Duration::from_millis(MIN_REQUEST_INTERVAL_MS);
    loop {
        let now = Instant::now();
        let last = {
            let guard = LAST_API_CALL.lock().unwrap();
            *guard
        };
        let elapsed = now.saturating_duration_since(last);
        if elapsed >= interval {
            *LAST_API_CALL.lock().unwrap() = now;
            return;
        }
        tokio::time::sleep(interval - elapsed).await;
    }
}
