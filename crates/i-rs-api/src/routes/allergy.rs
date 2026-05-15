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
        .route("/", get(list_allergys))
        .route("/", post(add_allergy))
        .route("/{id}", get(get_allergy))
        .route("/{id}", delete(delete_allergy))
}

#[derive(Debug, Deserialize)]
pub struct AddAllergyRequest {
    pub allergen: String,
    pub severity: String,
    pub symptoms: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_allergys(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.allergy.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_allergy(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddAllergyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let allergen = req.allergen;
    let severity = req.severity;
    let symptoms = req.symptoms.unwrap_or_default();
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_allergy::models::AllergyEntry::new(allergen, severity, symptoms, tags, remark);
    state.allergy.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_allergy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.allergy.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Allergy '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_allergy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.allergy.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Allergy '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
