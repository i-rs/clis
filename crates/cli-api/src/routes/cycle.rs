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
async fn update_cycle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.cycle.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Cycle '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_cycles))
        .route("/", post(add_cycle))
        .route("/{id}", get(get_cycle))
        .route("/{id}", delete(delete_cycle).patch(update_cycle))
}

#[derive(Debug, Deserialize)]
pub struct AddCycleRequest {
    pub date: serde_json::Value,
    pub event_type: String,
    pub symptoms: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_cycles(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.cycle.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_cycle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddCycleRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = serde_json::from_value(req.date)
        .map_err(|_| ApiError::BadRequest("Invalid date".to_string()))?;
    let event_type = req.event_type;
    let symptoms = req.symptoms.unwrap_or_default();
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_cycle::models::CycleEntry::new(date, event_type, symptoms, tags, remark);
    state.cycle.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_cycle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.cycle.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Cycle '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_cycle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.cycle.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Cycle '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
