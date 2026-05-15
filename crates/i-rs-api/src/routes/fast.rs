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
async fn update_fast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.fast.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Fast '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_fasts))
        .route("/", post(add_fast))
        .route("/{id}", get(get_fast))
        .route("/{id}", delete(delete_fast).patch(update_fast))
}

#[derive(Debug, Deserialize)]
pub struct AddFastRequest {
    pub start_time: String,
    pub end_time: Option<String>,
    pub target_hours: i32,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_fasts(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.fast.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_fast(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddFastRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let start_time = chrono::DateTime::parse_from_rfc3339(&req.start_time)
        .map_err(|_| ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string()))?
        .with_timezone(&chrono::Utc);
    let end_time = req.end_time.map(|s| chrono::DateTime::parse_from_rfc3339(&s)
        .map_err(|_| ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string()))
        .map(|dt| dt.with_timezone(&chrono::Utc))
    ).transpose()?;
    let target_hours = req.target_hours;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_fast::models::FastEntry::new(start_time, end_time, target_hours, tags, remark);
    state.fast.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_fast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.fast.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Fast '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_fast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.fast.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Fast '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
