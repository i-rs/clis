use std::sync::LazyLock;
use tokio::sync::Mutex;
use std::time::Instant;

use crate::mcp::McpManager;
use crate::pty::PtyManager;
use crate::lsp::LspSession;

pub static MCP_MANAGER: LazyLock<McpManager> = LazyLock::new(McpManager::new);
pub static PTY_MANAGER: LazyLock<PtyManager> = LazyLock::new(PtyManager::new);
pub static LSP_SESSION: LazyLock<Mutex<LspSession>> = LazyLock::new(|| Mutex::new(LspSession::new()));
pub static LAST_WEB_REQUEST: LazyLock<Mutex<Instant>> = LazyLock::new(|| Mutex::new(Instant::now()));
