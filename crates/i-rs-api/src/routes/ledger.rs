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
        .route("/", get(list_ledgers))
        .route("/", post(add_ledger))
        .route("/{id}", get(get_ledger))
        .route("/{id}", delete(delete_ledger))
}

#[derive(Debug, Deserialize)]
pub struct AddLedgerEntryRequest {
    pub date: String,
    pub amount: f64,
    pub currency: String,
    pub entry_type: String,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_ledgers(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.ledger.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_ledger(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddLedgerEntryRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;
    let amount = req.amount;
    let currency = req.currency;
    let entry_type = req.entry_type;
    let category = req.category;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_ledger::models::LedgerEntry {
        id: id.clone(),
        date,
        amount,
        currency,
        entry_type,
        category,
        tags,
        remark,
        created_at: now,
    };
    state.ledger.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_ledger(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.ledger.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Ledger '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_ledger(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.ledger.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Ledger '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
