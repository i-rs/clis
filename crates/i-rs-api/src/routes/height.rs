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
async fn update_height(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry_date = chrono::NaiveDate::parse_from_str(&id, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest(format!("Invalid date '{id}', expected YYYY-MM-DD")))?;
    let entry = state.height.write(|store| -> Result<_, ApiError> {
        let entry = store
            .records
            .get_mut(&entry_date)
            .ok_or_else(|| ApiError::NotFound(format!("Height '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok::<_, ApiError>(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_heights))
        .route("/", post(add_height))
        .route("/{id}", get(get_height))
        .route("/{id}", delete(delete_height).patch(update_height))
}

#[derive(Debug, Deserialize)]
pub struct AddHeightRecordRequest {
    pub date: String,
    pub height_cm: f64,
    pub weight_kg: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_heights(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.height.read(|store| {
        let entries: Vec<_> = store.records.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_height(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddHeightRecordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d").map_err(|_| {
        ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string())
    })?;
    let height_cm = req.height_cm;
    let weight_kg = req.weight_kg;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_height::models::HeightRecord {
        date,
        height_cm,
        weight_kg,
        tags,
        remark,
        created_at: now,
    };
    state.height.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_height(
    State(state): State<Arc<AppState>>,
    Path(id): Path<chrono::NaiveDate>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.height.read(|store| {
        store
            .records
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Height '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_height(
    State(state): State<Arc<AppState>>,
    Path(id): Path<chrono::NaiveDate>,
) -> ApiResult<Json<serde_json::Value>> {
    state.height.write(|store| {
        if store.records.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Height '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
