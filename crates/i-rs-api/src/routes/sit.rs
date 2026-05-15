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
async fn update_sit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.sit.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Sit '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_sits))
        .route("/", post(add_sit))
        .route("/{id}", get(get_sit))
        .route("/{id}", delete(delete_sit).put(update_sit))
}

#[derive(Debug, Deserialize)]
pub struct AddSitRequest {
    pub duration_minutes: i32,
    pub started_at: String,
    pub ended_at: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_sits(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.sit.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_sit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSitRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let duration_minutes = req.duration_minutes;
    let started_at = chrono::DateTime::parse_from_rfc3339(&req.started_at)
        .map_err(|_| ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string()))?
        .with_timezone(&chrono::Utc);
    let ended_at = chrono::DateTime::parse_from_rfc3339(&req.ended_at)
        .map_err(|_| ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string()))?
        .with_timezone(&chrono::Utc);
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_sit::models::SitEntry::new(duration_minutes, started_at, ended_at, tags, remark);
    state.sit.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_sit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.sit.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Sit '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_sit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.sit.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Sit '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
