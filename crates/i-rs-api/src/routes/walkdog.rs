use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_walkdogs))
        .route("/", post(add_walkdog))
        .route("/{id}", get(get_walkdog))
        .route("/{id}", delete(delete_walkdog))
}

#[derive(Debug, Deserialize)]
pub struct AddWalkdogRequest {
    pub dog_name: String,
    pub duration_minutes: i32,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_walkdogs(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.walkdog.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_walkdog(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddWalkdogRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let dog_name = req.dog_name;
    let duration_minutes = req.duration_minutes;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_walkdog::models::WalkdogEntry::new(dog_name, duration_minutes, tags, remark);
    state.walkdog.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_walkdog(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.walkdog.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Walkdog '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_walkdog(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.walkdog.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Walkdog '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
