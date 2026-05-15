use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use crate::api::{ok_json, ok_json_list};
use crate::response::{ApiError, ApiResult};
use crate::AppState;
use i_rs_core::parse_date;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_weights))
        .route("/", post(add_weight))
        .route("/stats", get(weight_stats))
        .route("/:date", get(get_weight))
}

#[derive(Debug, Deserialize)]
pub struct AddWeightRequest {
    pub date: Option<String>,
    pub weight: f64,
    pub remark: Option<Vec<String>>,
}

async fn list_weights(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.weight.read(|store| {
        store.records.values().cloned().collect::<Vec<_>>()
    });
    Ok(ok_json_list(records))
}

async fn add_weight(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddWeightRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = req.date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let parsed_date = parse_date(&date)?;
    let weight = req.weight;
    let remark = req.remark.unwrap_or_default();

    let exists = state.weight.read(|store| store.records.contains_key(&parsed_date));
    if exists {
        return Err(ApiError::Conflict(format!("Record for {date} already exists")));
    }

    let record = state.weight.write(|store| {
        let record = i_rs_weight::models::WeightRecord {
            date: parsed_date,
            weight,
            tags: Vec::new(),
            remark,
        };
        store.add_entry(record.clone());
        record
    });
    Ok(ok_json(record))
}

async fn get_weight(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let parsed_date = parse_date(&date)?;
    let record = state
        .weight
        .read(|store| store.records.get(&parsed_date).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("No record found for {date}")))?;
    Ok(ok_json(record))
}

async fn weight_stats(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let count = state.weight.read(|store| store.records.len());
    Ok(ok_json(serde_json::json!({ "count": count })))
}
