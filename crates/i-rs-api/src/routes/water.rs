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
async fn update_water(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.water.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Water '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_waters))
        .route("/", post(add_water))
        .route("/{id}", get(get_water))
        .route("/{id}", delete(delete_water).put(update_water))
}

#[derive(Debug, Deserialize)]
pub struct AddWaterRequest {
    pub amount_ml: i32,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_waters(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.water.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_water(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddWaterRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let amount_ml = req.amount_ml;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_water::models::WaterEntry::new(amount_ml, tags, remark);
    state.water.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_water(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.water.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Water '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_water(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.water.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Water '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
