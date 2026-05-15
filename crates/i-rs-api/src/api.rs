use axum::Json;

/// Wrap any serializable value into a success JSON response.
pub fn ok_json<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "success": true, "data": data }))
}

/// Wrap a list into a success JSON response with count metadata.
pub fn ok_json_list<T: serde::Serialize>(data: Vec<T>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "data": data,
        "meta": { "count": data.len() }
    }))
}

/// Wrap a unit success response.
pub fn ok_json_message() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "success": true, "message": "OK" }))
}
