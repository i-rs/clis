use serde::Serialize;

pub const MAX_TOOL_OUTPUT_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct ToolError {
    pub kind: ErrorKind,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    Timeout,
    InvalidArgs,
    ExternalError,
    InternalError,
    FileTooLarge,
    NetworkError,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "NOT_FOUND"),
            Self::PermissionDenied => write!(f, "PERMISSION_DENIED"),
            Self::Timeout => write!(f, "TIMEOUT"),
            Self::InvalidArgs => write!(f, "INVALID_ARGS"),
            Self::ExternalError => write!(f, "EXTERNAL_ERROR"),
            Self::InternalError => write!(f, "INTERNAL_ERROR"),
            Self::FileTooLarge => write!(f, "FILE_TOO_LARGE"),
            Self::NetworkError => write!(f, "NETWORK_ERROR"),
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for ToolError {}

impl ToolError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::NotFound,
            message: msg.into(),
            path: None,
        }
    }
    pub fn not_found_path(path: impl Into<String>) -> Self {
        let p = path.into();
        Self {
            kind: ErrorKind::NotFound,
            message: format!("not found: {}", p),
            path: Some(p),
        }
    }
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::PermissionDenied,
            message: msg.into(),
            path: None,
        }
    }
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Timeout,
            message: msg.into(),
            path: None,
        }
    }
    pub fn invalid_args(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::InvalidArgs,
            message: msg.into(),
            path: None,
        }
    }
    pub fn external(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::ExternalError,
            message: msg.into(),
            path: None,
        }
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::InternalError,
            message: msg.into(),
            path: None,
        }
    }
    pub fn file_too_large(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::FileTooLarge,
            message: msg.into(),
            path: None,
        }
    }
    pub fn network(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::NetworkError,
            message: msg.into(),
            path: None,
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.to_string())
    }
}

pub fn truncate_output(s: &str) -> String {
    crate::utils::truncate_output(s, MAX_TOOL_OUTPUT_BYTES)
}

impl From<anyhow::Error> for ToolError {
    fn from(e: anyhow::Error) -> Self {
        let msg = e.to_string();
        if msg.contains("not found")
            || msg.contains("No such file")
            || msg.contains("No such file or directory")
        {
            Self::not_found(msg)
        } else if msg.contains("Permission denied") || msg.contains("Access denied") {
            Self::permission_denied(msg)
        } else if msg.contains("timed out") || msg.contains("timeout") || msg.contains("deadline") {
            Self::timeout(msg)
        } else if msg.contains("invalid") || msg.contains("Invalid") {
            Self::invalid_args(msg)
        } else {
            Self::external(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_unchanged() {
        let s = "hello world";
        assert_eq!(truncate_output(s), s);
    }

    #[test]
    fn test_truncate_exact_limit() {
        let s = "a".repeat(MAX_TOOL_OUTPUT_BYTES);
        assert_eq!(truncate_output(&s).len(), s.len());
    }

    #[test]
    fn test_truncate_over_limit() {
        let s = "a".repeat(MAX_TOOL_OUTPUT_BYTES + 1000);
        let result = truncate_output(&s);
        assert!(result.len() < s.len());
        assert!(result.contains("[output truncated"));
    }

    #[test]
    fn test_truncate_preserves_newline() {
        let s = "a\n".repeat(MAX_TOOL_OUTPUT_BYTES / 2 + 100);
        let result = truncate_output(&s);
        assert!(result.contains("[output truncated"));
        assert!(!result.ends_with('a'));
    }

    #[test]
    fn test_error_display_format() {
        let e = ToolError::not_found("file.txt");
        let s = e.to_string();
        assert!(s.contains("NOT_FOUND"));
        assert!(s.contains("file.txt"));
    }

    #[test]
    fn test_error_json_format() {
        let e = ToolError::not_found_path("/etc/shadow");
        let json = e.to_json_string();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["kind"], "NOT_FOUND");
        assert_eq!(parsed["path"], "/etc/shadow");
    }

    #[test]
    fn test_error_with_path() {
        let e = ToolError::not_found_path("src/main.rs");
        assert_eq!(e.path.as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn test_from_anyhow_not_found() {
        let err = anyhow::anyhow!("file not found: foo.rs");
        let te: ToolError = err.into();
        assert_eq!(te.kind, ErrorKind::NotFound);
    }

    #[test]
    fn test_from_anyhow_permission() {
        let err = anyhow::anyhow!("Permission denied: /etc/passwd");
        let te: ToolError = err.into();
        assert_eq!(te.kind, ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_from_anyhow_timeout() {
        let err = anyhow::anyhow!("request timed out after 30s");
        let te: ToolError = err.into();
        assert_eq!(te.kind, ErrorKind::Timeout);
    }

    #[test]
    fn test_from_anyhow_generic() {
        let err = anyhow::anyhow!("something went wrong");
        let te: ToolError = err.into();
        assert_eq!(te.kind, ErrorKind::ExternalError);
    }
}
