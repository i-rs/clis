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
        .route("/", get(list_cyclings))
        .route("/", post(add_cycling))
        .route("/{id}", get(get_cycling))
        .route("/{id}", delete(delete_cycling))
}

#[derive(Debug, Deserialize)]
pub struct AddCyclingRequest {
    pub date: String,
    pub distance_km: f64,
    pub duration_minutes: u32,
    pub elevation_gain: Option<f64>,
    pub route: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_cyclings(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.cycling.read(|store| {
        let entries: Vec<_> = store.records.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_cycling(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddCyclingRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;
    let distance_km = req.distance_km;
    let duration_minutes = req.duration_minutes;
    let elevation_gain = req.elevation_gain;
    let route = req.route;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_cycling::models::CyclingRecord::new(date, distance_km, duration_minutes, elevation_gain, route, tags, remark);
    state.cycling.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_cycling(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.cycling.read(|store| {
        store.records.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Cycling '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_cycling(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.cycling.write(|store| {
        if store.records.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Cycling '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
