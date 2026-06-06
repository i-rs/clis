//! Unified error type for the i-rs-claw tool system.
//!
//! Replaces the `Result<String, String>` anti-pattern throughout the tools
//! and MCP layers with a structured `ClawError` enum. Provides type-safe
//! error categorization and consistent error formatting.

/// Error category for structured retry/validation decisions.
///
/// Replaces fragile string-prefix checks (`result.starts_with("错误:")`) with
/// a type-safe enum that all consumers (retry logic, result validation, context
/// compression scoring) can pattern-match on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Missing required parameter or invalid argument.
    Validation,
    /// Tool execution failure (CLI crash, IO error, etc.).
    Execution,
    /// MCP server communication error.
    Mcp,
    /// Operation timed out — safe to retry.
    Timeout,
    /// Network or API call failure — may be transient.
    Network,
    /// Requested resource not found — retrying won't help.
    NotFound,
    /// Tool returned an empty result (validation issue).
    EmptyResult,
    /// JSON output from tool is malformed or contains error fields.
    BadOutput,
    /// Generic / uncategorized error.
    Unknown,
}

impl ErrorCategory {
    /// Whether errors in this category are worth retrying.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout | Self::Network | Self::Execution)
    }
}

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
    #[allow(dead_code)]
    Timeout(String),
    /// Network or API call failure.
    #[allow(dead_code)]
    Network(String),
    /// Requested resource not found.
    NotFound(String),
    /// Generic error message (fallback for conversions).
    Message(String),
}

impl ClawError {
    #[allow(dead_code)]
    pub fn category(&self) -> ErrorCategory {
        match self {
            ClawError::Validation(_) => ErrorCategory::Validation,
            ClawError::Execution(_) => ErrorCategory::Execution,
            ClawError::Mcp(_) => ErrorCategory::Mcp,
            ClawError::Timeout(_) => ErrorCategory::Timeout,
            ClawError::Network(_) => ErrorCategory::Network,
            ClawError::NotFound(_) => ErrorCategory::NotFound,
            ClawError::Message(_) => ErrorCategory::Unknown,
        }
    }
}

/// Infer category from a tool result string (used for results returned as plain strings).
pub fn category_from_result(result: &str) -> ErrorCategory {
    if result.is_empty() {
        return ErrorCategory::EmptyResult;
    }
    if result.starts_with("参数错误:") || result.starts_with("错误: 未知工具") {
        return ErrorCategory::Validation;
    }
    if result.starts_with("执行错误:") || result.starts_with("错误:") {
        return ErrorCategory::Execution;
    }
    if result.starts_with("MCP 错误:") {
        return ErrorCategory::Mcp;
    }
    if result.starts_with("超时:") || result.contains("执行超时") {
        return ErrorCategory::Timeout;
    }
    if result.starts_with("网络错误:") {
        return ErrorCategory::Network;
    }
    if result.starts_with("未找到:") {
        return ErrorCategory::NotFound;
    }
    if result.starts_with("Error") || result.starts_with("error") {
        return ErrorCategory::Execution;
    }
    ErrorCategory::Unknown
}

/// Check if a tool result string indicates failure (backward-compatible helper).
#[allow(dead_code)]
pub fn is_error_result(result: &str) -> bool {
    category_from_result(result).is_retryable_or_fatal()
}

impl ErrorCategory {
    /// Returns true for any error category (i.e. not Unknown/EmptyResult when result is fine).
    pub fn is_error(&self) -> bool {
        !matches!(self, ErrorCategory::Unknown)
    }

    /// Returns true if this category represents any kind of failure.
    /// Used as a replacement for `result.starts_with("错误:")`.
    pub fn is_retryable_or_fatal(&self) -> bool {
        matches!(
            self,
            ErrorCategory::Validation
                | ErrorCategory::Execution
                | ErrorCategory::Mcp
                | ErrorCategory::Timeout
                | ErrorCategory::Network
                | ErrorCategory::NotFound
                | ErrorCategory::EmptyResult
                | ErrorCategory::BadOutput
        )
    }
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

impl From<anyhow::Error> for ClawError {
    fn from(e: anyhow::Error) -> Self {
        ClawError::Execution(format!("{}", e))
    }
}
