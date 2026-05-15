use axum::{
    response::{Json, IntoResponse},
    http::StatusCode,
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
        (status, Json(serde_json::json!({
            "success": false,
            "error": {
                "code": code_str,
                "message": self.to_string()
            },
            "meta": ApiMeta::new()
        })))
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
        } else if lower.contains("invalid") || lower.contains("parse") || lower.contains("validation") {
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
