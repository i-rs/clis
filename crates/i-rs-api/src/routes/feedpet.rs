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
async fn update_feedpet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.feedpet.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Feedpet '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_feedpets))
        .route("/", post(add_feedpet))
        .route("/{id}", get(get_feedpet))
        .route("/{id}", delete(delete_feedpet).patch(update_feedpet))
}

#[derive(Debug, Deserialize)]
pub struct AddFeedpetRequest {
    pub pet_name: String,
    pub food_type: String,
    pub amount: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_feedpets(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.feedpet.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_feedpet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddFeedpetRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let pet_name = req.pet_name;
    let food_type = req.food_type;
    let amount = req.amount;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_feedpet::models::FeedpetEntry::new(pet_name, food_type, amount, tags, remark);
    state.feedpet.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_feedpet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.feedpet.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Feedpet '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_feedpet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.feedpet.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Feedpet '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
