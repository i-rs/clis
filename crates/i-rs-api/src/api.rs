use axum::Json;
use axum::http::StatusCode;

/// Call a sync service function that returns data, wrapped in spawn_blocking.
/// Returns JSON: `{ "success": true, "data": <result> }`
pub async fn call_service<F, T>(f: F) -> Result<Json<serde_json::Value>, (StatusCode, String)>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: serde::Serialize + Send + 'static,
{
    let data = tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Server error: {e}")))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let value = serde_json::to_value(data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialize error: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true, "data": value })))
}

/// Call a sync service function that returns `()`, wrapped in spawn_blocking.
/// Returns JSON: `{ "success": true, "message": "OK" }`
pub async fn call_service_unit<F>(f: F) -> Result<Json<serde_json::Value>, (StatusCode, String)>
where
    F: FnOnce() -> anyhow::Result<()> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Server error: {e}")))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "success": true, "message": "OK" })))
}
