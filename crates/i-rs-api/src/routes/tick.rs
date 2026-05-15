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
        .route("/", get(list_ticks))
        .route("/", post(add_tick))
        .route("/{id}", get(get_tick))
        .route("/{id}", delete(delete_tick))
}

#[derive(Debug, Deserialize)]
pub struct AddTickRequest {
    pub task_name: String,
    pub duration_seconds: i64,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub started_at: String,
    pub ended_at: String,
}

async fn list_ticks(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.tick.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_tick(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddTickRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let task_name = req.task_name;
    let duration_seconds = req.duration_seconds;
    let description = req.description;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let started_at = chrono::DateTime::parse_from_rfc3339(&req.started_at)
        .map_err(|_| ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string()))?
        .with_timezone(&chrono::Utc);
    let ended_at = chrono::DateTime::parse_from_rfc3339(&req.ended_at)
        .map_err(|_| ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string()))?
        .with_timezone(&chrono::Utc);
    let entry = i_rs_tick::models::TickEntry::new(task_name, duration_seconds, description, tags, remark, started_at, ended_at);
    state.tick.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_tick(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.tick.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Tick '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_tick(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.tick.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Tick '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
