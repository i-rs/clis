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
async fn update_towel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.towel.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Towel '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_towels))
        .route("/", post(add_towel))
        .route("/{id}", get(get_towel))
        .route("/{id}", delete(delete_towel).put(update_towel))
}

#[derive(Debug, Deserialize)]
pub struct AddTowelRequest {
    pub towel_type: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_towels(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.towel.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_towel(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddTowelRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let towel_type = req.towel_type;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_towel::models::TowelEntry::new(towel_type, tags, remark);
    state.towel.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_towel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.towel.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Towel '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_towel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.towel.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Towel '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
