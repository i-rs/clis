use axum::{
    response::{Json, IntoResponse},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ApiMeta>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

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

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ApiMeta::new()),
        })
    }
}

impl ApiResponse<String> {
    pub fn server_error(message: &str) -> impl IntoResponse {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Self {
                success: false,
                data: None,
                error: Some(ApiError {
                    code: "SERVER_ERROR".to_string(),
                    message: message.to_string(),
                }),
                meta: Some(ApiMeta::new()),
            }),
        )
    }

    pub fn not_found(message: &str) -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Json(Self {
                success: false,
                data: None,
                error: Some(ApiError {
                    code: "NOT_FOUND".to_string(),
                    message: message.to_string(),
                }),
                meta: Some(ApiMeta::new()),
            }),
        )
    }

    pub fn bad_request(message: &str) -> impl IntoResponse {
        (
            StatusCode::BAD_REQUEST,
            Json(Self {
                success: false,
                data: None,
                error: Some(ApiError {
                    code: "BAD_REQUEST".to_string(),
                    message: message.to_string(),
                }),
                meta: Some(ApiMeta::new()),
            }),
        )
    }
}

impl fmt::Display for ApiResponse<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}
