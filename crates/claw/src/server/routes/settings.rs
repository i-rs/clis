use crate::server::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::Value;

/// GET /api/settings — List all app settings.
pub async fn list_settings(State(state): State<AppState>) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    match core.config_store.app_settings.load_all().await {
        Ok(rows) => {
            let result: Vec<Value> = rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "key": r.key,
                        "value": r.value,
                        "updated_at": r.updated_at,
                    })
                })
                .collect();
            super::ApiResponse::ok(result)
        }
        Err(e) => super::ApiResponse::err(&format!("Failed to list settings: {}", e)),
    }
}

/// PUT /api/settings — Set a setting. Body: {key: "...", value: ...}
pub async fn set_setting(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let key = match body.get("key").and_then(|v| v.as_str()) {
        Some(k) if !k.is_empty() => k,
        _ => return super::ApiResponse::err("Missing or invalid 'key' field"),
    };

    let value = match body.get("value") {
        Some(v) => v,
        None => return super::ApiResponse::err("Missing 'value' field"),
    };

    let core = state.core.read().await;
    match core.config_store.app_settings.set(key, value).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "key": key,
            "status": "set",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to set setting: {}", e)),
    }
}

/// GET /api/settings/{key} — Get a single setting.
pub async fn get_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    match core.config_store.app_settings.get(&key).await {
        Ok(Some(value)) => super::ApiResponse::ok(value),
        Ok(None) => super::ApiResponse::err(&format!("Setting '{}' not found", key)),
        Err(e) => super::ApiResponse::err(&format!("Failed to get setting: {}", e)),
    }
}

/// PUT /api/settings/{key} — Set/update a single setting. Body is the value.
pub async fn set_setting_by_key(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    match core.config_store.app_settings.set(&key, &body).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "key": key,
            "status": "set",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to set setting: {}", e)),
    }
}

/// DELETE /api/settings/{key} — Delete a setting.
pub async fn delete_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;

    // Delete is done by setting an empty value pattern.
    // ConfigStore doesn't have a delete for settings, so we use delete on the repo.
    // Actually the trait only has get/set/load_all. We'll set to null as a workaround.
    match core
        .config_store
        .app_settings
        .set(&key, &serde_json::Value::Null)
        .await
    {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "key": key,
            "status": "deleted",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to delete setting: {}", e)),
    }
}
