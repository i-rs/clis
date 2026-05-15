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
async fn update_vision(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry_date = chrono::NaiveDate::parse_from_str(&id, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest(format!("Invalid date '{id}', expected YYYY-MM-DD")))?;
    let entry = state.vision.write(|store| -> Result<_, ApiError> {
        let entry = store.records.get_mut(&entry_date)
            .ok_or_else(|| ApiError::NotFound(format!("Vision '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok::<_, ApiError>(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_visions))
        .route("/", post(add_vision))
        .route("/{id}", get(get_vision))
        .route("/{id}", delete(delete_vision).put(update_vision))
}

#[derive(Debug, Deserialize)]
pub struct AddVisionRecordRequest {
    pub date: String,
    pub left_sphere: Option<f64>,
    pub right_sphere: Option<f64>,
    pub left_cylinder: Option<f64>,
    pub right_cylinder: Option<f64>,
    pub left_axis: Option<i32>,
    pub right_axis: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_visions(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.vision.read(|store| {
        let entries: Vec<_> = store.records.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_vision(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddVisionRecordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;
    let left_sphere = req.left_sphere;
    let right_sphere = req.right_sphere;
    let left_cylinder = req.left_cylinder;
    let right_cylinder = req.right_cylinder;
    let left_axis = req.left_axis;
    let right_axis = req.right_axis;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_vision::models::VisionRecord {
        date,
        left_sphere,
        right_sphere,
        left_cylinder,
        right_cylinder,
        left_axis,
        right_axis,
        tags,
        remark,
        created_at: now,
    };
    state.vision.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_vision(
    State(state): State<Arc<AppState>>,
    Path(id): Path<chrono::NaiveDate>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.vision.read(|store| {
        store.records.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Vision '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_vision(
    State(state): State<Arc<AppState>>,
    Path(id): Path<chrono::NaiveDate>,
) -> ApiResult<Json<serde_json::Value>> {
    state.vision.write(|store| {
        if store.records.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Vision '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
