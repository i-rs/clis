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
async fn update_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.run.write(|store| -> Result<_, ApiError> {
        let entry = store.records.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Run '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_runs))
        .route("/", post(add_run))
        .route("/{id}", get(get_run))
        .route("/{id}", delete(delete_run).patch(update_run))
}

#[derive(Debug, Deserialize)]
pub struct AddRunRecordRequest {
    pub date: String,
    pub distance_km: f64,
    pub duration_minutes: f64,
    pub pace: String,
    pub heart_rate: Option<u32>,
    pub weather: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_runs(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.run.read(|store| {
        let entries: Vec<_> = store.records.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_run(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddRunRecordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;
    let distance_km = req.distance_km;
    let duration_minutes = req.duration_minutes;
    let pace = req.pace;
    let heart_rate = req.heart_rate;
    let weather = req.weather;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_run::models::RunRecord {
        id: id.clone(),
        date,
        distance_km,
        duration_minutes,
        pace,
        heart_rate,
        weather,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.run.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.run.read(|store| {
        store.records.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Run '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.run.write(|store| {
        if store.records.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Run '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
