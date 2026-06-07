use axum::Json;

/// Health check endpoint.
pub async fn health() -> Json<super::ApiResponse<&'static str>> {
    super::ApiResponse::ok("OK")
}
