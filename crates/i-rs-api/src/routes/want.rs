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
        .route("/", get(list_wants))
        .route("/", post(add_want))
        .route("/{id}", get(get_want))
        .route("/{id}", delete(delete_want))
}

#[derive(Debug, Deserialize)]
pub struct AddWantRequest {
    pub name: String,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub priority: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_wants(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.want.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_want(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddWantRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let url = req.url;
    let price = req.price;
    let currency = req.currency;
    let priority = req.priority;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_want::models::WantEntry::new(name, url, price, currency, priority, tags, remark);
    state.want.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_want(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.want.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Want '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_want(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.want.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Want '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
