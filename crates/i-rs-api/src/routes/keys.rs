use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_keys))
        .route("/", post(add_key))
        .route("/:name", get(get_key))
        .route("/:name", delete(delete_key))
}

#[derive(Debug, Deserialize)]
pub struct AddKeyRequest {
    pub value: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListKeysQuery {
    pub search: Option<String>,
}

async fn list_keys(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListKeysQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let records: Vec<i_rs_keys::models::ListItem> = state.keys.read(|store| {
        if let Some(ref search) = query.search {
            store
                .entries
                .values()
                .filter(|e| {
                    e.name.contains(search)
                        || e.key_type.contains(search)
                        || e.tags.contains(search)
                })
                .map(|e| e.into())
                .collect()
        } else {
            store.entries.values().map(|e| e.into()).collect()
        }
    });
    Ok(ok_json_list(records))
}

async fn add_key(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<AddKeyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let value = req.value;
    let remark = req.remark.unwrap_or_default();

    let exists = state.keys.read(|store| store.entries.contains_key(&name));
    if exists {
        return Err(ApiError::Conflict(format!("Key '{name}' already exists")));
    }

    let now = chrono::Utc::now();
    let entry = state.keys.write(|store| {
        let entry = i_rs_keys::models::KeyEntry {
            name: name.clone(),
            key_type: "api_key".to_string(),
            tags: Vec::new(),
            remark: vec![remark],
            created_at: now,
            updated_at: now,
        };
        store.add_entry(entry.clone());
        entry
    });
    // Store the actual value in keychain (can't avoid sync I/O here)
    let _ = i_rs_keys::storage::store_key(&name, &value);
    Ok(ok_json(entry))
}

async fn get_key(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state
        .keys
        .read(|store| store.entries.get(&name).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("Key '{name}' not found")))?;
    Ok(ok_json(entry))
}

async fn delete_key(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.keys.write(|store| {
        store
            .entries
            .remove(&name)
            .ok_or_else(|| anyhow::anyhow!("Key '{name}' not found"))
    })?;
    let _ = i_rs_keys::storage::delete_key(&name);
    Ok(ok_json_message())
}
