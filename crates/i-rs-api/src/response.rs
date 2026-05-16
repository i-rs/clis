use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::fmt;

/// Typed API error with proper HTTP status code mapping.
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl ApiError {
    pub fn into_response(self) -> (StatusCode, Json<serde_json::Value>) {
        let (code_str, status) = match &self {
            ApiError::NotFound(_) => ("NOT_FOUND", StatusCode::NOT_FOUND),
            ApiError::BadRequest(_) => ("BAD_REQUEST", StatusCode::BAD_REQUEST),
            ApiError::Conflict(_) => ("CONFLICT", StatusCode::CONFLICT),
            ApiError::Internal(_) => ("SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR),
        };
        (
            status,
            Json(serde_json::json!({
                "success": false,
                "error": {
                    "code": code_str,
                    "message": self.to_string()
                },
                "meta": ApiMeta::new()
            })),
        )
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        let msg = e.to_string();
        let lower = msg.to_lowercase();
        if lower.contains("not found") || lower.starts_with("no ") {
            ApiError::NotFound(msg)
        } else if lower.contains("already exists") {
            ApiError::Conflict(msg)
        } else if lower.contains("invalid")
            || lower.contains("parse")
            || lower.contains("validation")
        {
            ApiError::BadRequest(msg)
        } else {
            ApiError::Internal(msg)
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg)
            | ApiError::BadRequest(msg)
            | ApiError::Conflict(msg)
            | ApiError::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        self.into_response().into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Serialize)]
pub struct ApiMeta {
    pub timestamp: String,
    pub version: String,
}

impl ApiMeta {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for ApiMeta {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_api_error_not_found_display() {
        let err = ApiError::NotFound("test not found".to_string());
        assert_eq!(err.to_string(), "test not found");
    }

    #[test]
    fn test_api_error_bad_request_display() {
        let err = ApiError::BadRequest("bad input".to_string());
        assert_eq!(err.to_string(), "bad input");
    }

    #[test]
    fn test_api_error_not_found_status() {
        let err = ApiError::NotFound("missing".to_string());
        let (status, json) = err.into_response();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json.0["success"], false);
        assert_eq!(json.0["error"]["code"], "NOT_FOUND");
    }

    #[test]
    fn test_api_error_bad_request_status() {
        let err = ApiError::BadRequest("invalid".to_string());
        let (status, _json) = err.into_response();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_conflict_status() {
        let err = ApiError::Conflict("exists".to_string());
        let (status, _json) = err.into_response();
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[test]
    fn test_api_error_internal_status() {
        let err = ApiError::Internal("server error".to_string());
        let (status, _json) = err.into_response();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_from_anyhow_not_found() {
        let e = anyhow::anyhow!("record not found");
        let api_err: ApiError = e.into();
        assert!(matches!(api_err, ApiError::NotFound(_)));
    }

    #[test]
    fn test_from_anyhow_conflict() {
        let e = anyhow::anyhow!("item already exists");
        let api_err: ApiError = e.into();
        assert!(matches!(api_err, ApiError::Conflict(_)));
    }

    #[test]
    fn test_from_anyhow_bad_request() {
        let e = anyhow::anyhow!("invalid input");
        let api_err: ApiError = e.into();
        assert!(matches!(api_err, ApiError::BadRequest(_)));
    }

    #[test]
    fn test_from_anyhow_internal() {
        let e = anyhow::anyhow!("something went wrong");
        let api_err: ApiError = e.into();
        assert!(matches!(api_err, ApiError::Internal(_)));
    }

    #[test]
    fn test_api_meta_contains_timestamp_and_version() {
        let meta = ApiMeta::new();
        assert!(!meta.timestamp.is_empty());
        assert_eq!(meta.version, "0.0.2");
    }
}
