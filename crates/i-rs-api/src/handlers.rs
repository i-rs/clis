use axum::Json;
use serde_json::json;

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "status": "ok",
        "service": "i-rs-api",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "REST API for i-rs CLI tools",
        "endpoints": [
            "/api/todo",
            "/api/weight",
            "/api/habit",
            "/api/note",
            "/api/bookmark",
            "/api/mood",
            "/api/kv",
            "/api/keys"
        ]
    }))
}
