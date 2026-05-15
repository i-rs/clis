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
async fn update_cal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.cal.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Cal '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_cals))
        .route("/", post(add_cal))
        .route("/{id}", get(get_cal))
        .route("/{id}", delete(delete_cal).put(update_cal))
}

#[derive(Debug, Deserialize)]
pub struct AddCalRequest {
    pub food_name: String,
    pub calories: i32,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub date: serde_json::Value,
}

async fn list_cals(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.cal.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_cal(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddCalRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let food_name = req.food_name;
    let calories = req.calories;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let date = serde_json::from_value(req.date)
        .map_err(|_| ApiError::BadRequest(format!("Invalid date")))?;
    let entry = i_rs_cal::models::CalEntry::new(food_name, calories, tags, remark, date);
    state.cal.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_cal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.cal.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Cal '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_cal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.cal.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Cal '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
