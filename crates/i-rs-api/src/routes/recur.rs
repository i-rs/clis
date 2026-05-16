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
async fn update_recur(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.recur.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Recur '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_recurs))
        .route("/", post(add_recur))
        .route("/{id}", get(get_recur))
        .route("/{id}", delete(delete_recur).patch(update_recur))
}

#[derive(Debug, Deserialize)]
pub struct AddRecurEntryRequest {
    pub name: String,
    pub amount: f64,
    pub currency: String,
    pub frequency: String,
    pub start_date: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_recurs(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.recur.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_recur(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddRecurEntryRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let amount = req.amount;
    let currency = req.currency;
    let frequency = req.frequency;
    let parsed_start_date = chrono::DateTime::parse_from_rfc3339(&req.start_date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let start_date = parsed_start_date.with_timezone(&chrono::Utc);
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_recur::models::RecurEntry {
        name,
        amount,
        currency,
        frequency,
        start_date,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.recur.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_recur(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.recur.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Recur '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_recur(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.recur.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Recur '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
