#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ProviderError {
    RateLimited { retry_after_ms: Option<u64> },
    Timeout,
    ServerError { status: u16 },
    AuthFailed,
    ContextLengthExceeded,
    ConnectionFailed,
    Unknown(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited { .. } => write!(f, "rate_limited"),
            Self::Timeout => write!(f, "timeout"),
            Self::ServerError { status } => write!(f, "server_error_{}", status),
            Self::AuthFailed => write!(f, "auth_failed"),
            Self::ContextLengthExceeded => write!(f, "context_length_exceeded"),
            Self::ConnectionFailed => write!(f, "connection_failed"),
            Self::Unknown(msg) => write!(f, "{}", msg),
        }
    }
}

impl ProviderError {
    pub fn from_message(msg: &str) -> Self {
        let lower = msg.to_lowercase();
        if lower.contains("rate") || lower.contains("限流") || lower.contains("quota") {
            Self::RateLimited { retry_after_ms: None }
        } else if lower.contains("timeout") || lower.contains("timed out") {
            Self::Timeout
        } else if lower.contains("context_length") || lower.contains("max tokens") || lower.contains("token limit") || lower.contains("context window") {
            Self::ContextLengthExceeded
        } else if lower.contains("auth") || lower.contains("unauthorized") || lower.contains("401") || lower.contains("invalid api key") {
            Self::AuthFailed
        } else if lower.contains("connection") || lower.contains("econnreset") || lower.contains("econnrefused") {
            Self::ConnectionFailed
        } else if lower.contains("502") || lower.contains("503") || lower.contains("504") {
            Self::ServerError { status: 502 }
        } else {
            Self::Unknown(msg.to_string())
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited { .. } | Self::Timeout | Self::ServerError { .. } | Self::ConnectionFailed)
    }
}
