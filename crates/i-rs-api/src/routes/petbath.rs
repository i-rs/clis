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
        .route("/", get(list_petbaths))
        .route("/", post(add_petbath))
        .route("/{id}", get(get_petbath))
        .route("/{id}", delete(delete_petbath))
}

#[derive(Debug, Deserialize)]
pub struct AddPetbathRequest {
    pub pet_name: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_petbaths(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.petbath.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_petbath(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddPetbathRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let pet_name = req.pet_name;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_petbath::models::PetbathEntry::new(pet_name, tags, remark);
    state.petbath.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_petbath(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.petbath.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Petbath '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_petbath(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.petbath.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Petbath '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
