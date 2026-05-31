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

async fn update_weight(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.weight.write(|store| -> Result<_, ApiError> {
        let key = store
            .entries
            .iter()
            .find(|(k, _)| k.starts_with(&id))
            .map(|(k, _)| k.clone())
            .ok_or_else(|| ApiError::NotFound(format!("Weight '{id}' not found")))?;
        let entry = store
            .entries
            .get_mut(&key)
            .ok_or_else(|| ApiError::NotFound(format!("Weight '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_weights))
        .route("/", post(add_weight))
        .route("/stats", get(weight_stats))
        .route("/{id}", get(get_weight).patch(update_weight))
        .route("/{id}", delete(delete_weight))
}

#[derive(Debug, Deserialize)]
pub struct AddWeightRequest {
    pub date: Option<String>,
    pub weight: f64,
    pub tag: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_weights(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state
        .weight
        .read(|store| i_rs_weight::service::list_weights(store, None).map_err(ApiError::from))?;
    Ok(ok_json_list(records))
}

async fn add_weight(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddWeightRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let weight = req.weight;
    let tags = req.tag.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let record = state.weight.write(|store| {
        i_rs_weight::service::add_weight(store, req.date, weight, tags, remark)
            .map_err(ApiError::from)
    })?;
    Ok(ok_json(record))
}

async fn get_weight(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let record = state
        .weight
        .read(|store| i_rs_weight::service::get_weight(store, &id).map_err(ApiError::from))?;
    Ok(ok_json(record))
}

async fn weight_stats(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let count = state.weight.read(|store| store.entries.len());
    Ok(ok_json(serde_json::json!({"count": count})))
}

async fn delete_weight(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .weight
        .write(|store| i_rs_weight::service::delete_weight(store, id).map_err(ApiError::from))?;
    Ok(ok_json_message())
}
