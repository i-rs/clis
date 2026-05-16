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
async fn update_sub(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.sub.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Sub '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_subs))
        .route("/", post(add_sub))
        .route("/{id}", get(get_sub))
        .route("/{id}", delete(delete_sub).patch(update_sub))
}

#[derive(Debug, Deserialize)]
pub struct AddSubEntryRequest {
    pub name: String,
    pub amount: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub next_billing_date: String,
    pub url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_subs(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.sub.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_sub(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSubEntryRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let amount = req.amount;
    let currency = req.currency;
    let billing_cycle = req.billing_cycle;
    let parsed_next_billing_date = chrono::DateTime::parse_from_rfc3339(&req.next_billing_date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let next_billing_date = parsed_next_billing_date.with_timezone(&chrono::Utc);
    let url = req.url;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_sub::models::SubEntry {
        name,
        amount,
        currency,
        billing_cycle,
        next_billing_date,
        url,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.sub.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_sub(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.sub.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Sub '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_sub(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.sub.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Sub '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
