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
async fn update_step(
    State(state): State<Arc<AppState>>,
    Path(id): Path<chrono::NaiveDate>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.step.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Step '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_steps))
        .route("/", post(add_step))
        .route("/{id}", get(get_step))
        .route("/{id}", delete(delete_step).patch(update_step))
}

#[derive(Debug, Deserialize)]
pub struct AddStepRequest {
    pub steps: i32,
    pub distance: Option<f64>,
    pub date: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_steps(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.step.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_step(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddStepRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let steps = req.steps;
    let distance = req.distance;
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d").map_err(|_| {
        ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string())
    })?;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_step::models::StepEntry::new(steps, distance, date, tags, remark);
    state.step.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_step(
    State(state): State<Arc<AppState>>,
    Path(id): Path<chrono::NaiveDate>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.step.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Step '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_step(
    State(state): State<Arc<AppState>>,
    Path(id): Path<chrono::NaiveDate>,
) -> ApiResult<Json<serde_json::Value>> {
    state.step.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Step '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
