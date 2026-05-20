//! Unified error type for the i-rs-claw tool system.
//!
//! Replaces the `Result<String, String>` anti-pattern throughout the tools
//! and MCP layers with a structured `ClawError` enum. Provides type-safe
//! error categorization and consistent error formatting.

/// Unified error type for tool execution, MCP calls, and validation.
#[derive(Debug)]
pub enum ClawError {
    /// Missing required parameter or invalid argument.
    Validation(String),
    /// Tool execution failure (CLI, IO, filesystem, etc.).
    Execution(String),
    /// MCP server communication error.
    Mcp(String),
    /// Operation timed out.
    Timeout(String),
    /// Network or API call failure.
    #[allow(dead_code)]
    Network(String),
    /// Requested resource not found.
    NotFound(String),
    /// Generic error message (fallback for conversions).
    Message(String),
}

// ── Display ──

impl std::fmt::Display for ClawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClawError::Validation(msg) => write!(f, "参数错误: {}", msg),
            ClawError::Execution(msg) => write!(f, "执行错误: {}", msg),
            ClawError::Mcp(msg) => write!(f, "MCP 错误: {}", msg),
            ClawError::Timeout(msg) => write!(f, "超时: {}", msg),
            ClawError::Network(msg) => write!(f, "网络错误: {}", msg),
            ClawError::NotFound(msg) => write!(f, "未找到: {}", msg),
            ClawError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ClawError {}

// ── From conversions (for `?` operator convenience) ──

impl From<String> for ClawError {
    fn from(s: String) -> Self {
        ClawError::Message(s)
    }
}

impl From<&str> for ClawError {
    fn from(s: &str) -> Self {
        ClawError::Message(s.to_string())
    }
}

impl From<std::io::Error> for ClawError {
    fn from(e: std::io::Error) -> Self {
        ClawError::Execution(format!("IO 错误: {}", e))
    }
}
