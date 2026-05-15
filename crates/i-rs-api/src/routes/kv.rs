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
        .route("/", get(list_kv))
        .route("/:key", get(get_kv))
        .route("/:key", post(set_kv))
        .route("/:key", delete(delete_kv))
}

#[derive(Debug, Deserialize)]
pub struct SetKvRequest {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ListKvQuery {
    pub search: Option<String>,
}

async fn list_kv(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListKvQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let records: Vec<i_rs_kv::models::ListItem> = state.kv.read(|store| {
        if let Some(ref search) = query.search {
            store
                .entries
                .values()
                .filter(|e| {
                    e.key.contains(search)
                        || e.value.contains(search)
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

async fn set_kv(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<SetKvRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let value = req.value;
    let now = chrono::Utc::now();
    let entry = state.kv.write(|store| {
        let entry = i_rs_kv::models::KvEntry {
            key: key.clone(),
            value,
            tags: Vec::new(),
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        store.add_entry(entry.clone());
        entry
    });
    Ok(ok_json(entry))
}

async fn get_kv(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state
        .kv
        .read(|store| store.entries.get(&key).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("Key '{key}' not found")))?;
    Ok(ok_json(entry))
}

async fn delete_kv(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.kv.write(|store| {
        store
            .entries
            .remove(&key)
            .ok_or_else(|| anyhow::anyhow!("Key '{key}' not found"))
    })?;
    Ok(ok_json_message())
}
