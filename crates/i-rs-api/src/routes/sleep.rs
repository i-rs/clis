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
        .route("/", get(list_sleeps))
        .route("/", post(add_sleep))
        .route("/{id}", get(get_sleep))
        .route("/{id}", delete(delete_sleep))
}

#[derive(Debug, Deserialize)]
pub struct AddSleepRecordRequest {
    pub bedtime: String,
    pub wake_time: String,
    pub quality: i32,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_sleeps(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.sleep.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_sleep(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSleepRecordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let parsed_bedtime = chrono::DateTime::parse_from_rfc3339(&req.bedtime)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let bedtime = parsed_bedtime.with_timezone(&chrono::Utc);
    let parsed_wake_time = chrono::DateTime::parse_from_rfc3339(&req.wake_time)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let wake_time = parsed_wake_time.with_timezone(&chrono::Utc);
    let quality = req.quality;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_sleep::models::SleepRecord {
        id: id.clone(),
        bedtime,
        wake_time,
        quality,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.sleep.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_sleep(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.sleep.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Sleep '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_sleep(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.sleep.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Sleep '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
