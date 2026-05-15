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
        .route("/", get(list_gifts))
        .route("/", post(add_gift))
        .route("/{id}", get(get_gift))
        .route("/{id}", delete(delete_gift))
}

#[derive(Debug, Deserialize)]
pub struct AddGiftRequest {
    pub name: String,
    pub gift_type: String,
    pub recipient: String,
    pub occasion: String,
    pub value: f64,
    pub date: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_gifts(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.gift.read(|store| {
        let entries: Vec<_> = store.gifts.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_gift(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddGiftRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let gift_type: i_rs_gift::models::GiftType = serde_json::from_value(serde_json::json!(req.gift_type))
        .map_err(|e| ApiError::BadRequest(format!("Invalid gift_type: {e}")))?;
    let recipient = req.recipient;
    let occasion = req.occasion;
    let value = req.value;
    let date: chrono::DateTime<chrono::Utc> = req.date.parse()
        .map_err(|e| ApiError::BadRequest(format!("Invalid date: {e}")))?;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_gift::models::Gift {
        name,
        gift_type,
        recipient,
        occasion,
        value,
        date,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.gift.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_gift(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.gift.read(|store| {
        store.gifts.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Gift '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_gift(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.gift.write(|store| {
        if store.gifts.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Gift '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
