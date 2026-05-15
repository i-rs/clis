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
        i_rs_weight::service::list_weights(store, None).map_err(ApiError::from)
    })?;
    Ok(ok_json_list(records))
}

async fn add_weight(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddWeightRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = req.date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let weight = req.weight;
    let remark = req.remark.unwrap_or_default();
    let record = state.weight.write(|store| {
        i_rs_weight::service::add_weight(store, date, weight, remark).map_err(ApiError::from)
    })?;
    Ok(ok_json(record))
}

async fn get_weight(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let record = state.weight.read(|store| {
        i_rs_weight::service::get_weight(store, &date).map_err(ApiError::from)
    })?;
    Ok(ok_json(record))
}

async fn weight_stats(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let count = state.weight.read(|store| store.records.len());
    Ok(ok_json(serde_json::json!({ "count": count })))
}
