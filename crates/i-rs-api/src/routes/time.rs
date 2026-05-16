use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::AppState;
use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};
async fn update_time(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.time.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Time '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_times))
        .route("/", post(add_time))
        .route("/{id}", get(get_time))
        .route("/{id}", delete(delete_time).patch(update_time))
}

#[derive(Debug, Deserialize)]
pub struct AddTimeEntryRequest {
    pub name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_minutes: i64,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_times(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.time.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_time(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddTimeEntryRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let parsed_start_time = chrono::DateTime::parse_from_rfc3339(&req.start_time)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let start_time = parsed_start_time.with_timezone(&chrono::Utc);
    let end_time = req
        .end_time
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .transpose()?;
    let duration_minutes = req.duration_minutes;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_time::models::TimeEntry {
        id: id.clone(),
        name,
        start_time,
        end_time,
        duration_minutes,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.time.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_time(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.time.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Time '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_time(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.time.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Time '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
