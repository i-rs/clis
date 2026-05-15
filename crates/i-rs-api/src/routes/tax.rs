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
        .route("/", get(list_taxs))
        .route("/", post(add_tax))
        .route("/{id}", get(get_tax))
        .route("/{id}", delete(delete_tax))
}

#[derive(Debug, Deserialize)]
pub struct AddTaxRecordRequest {
    pub name: String,
    pub tax_type: String,
    pub amount: f64,
    pub date: String,
    pub year: i32,
    pub status: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_taxs(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.tax.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_tax(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddTaxRecordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let tax_type: i_rs_tax::models::TaxType = serde_json::from_value(serde_json::json!(req.tax_type))
        .map_err(|e| ApiError::BadRequest(format!("Invalid tax_type: {e}")))?;
    let amount = req.amount;
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;
    let year = req.year;
    let status: i_rs_tax::models::TaxStatus = serde_json::from_value(serde_json::json!(req.status))
        .map_err(|e| ApiError::BadRequest(format!("Invalid status: {e}")))?;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_tax::models::TaxRecord {
        name,
        tax_type,
        amount,
        date,
        year,
        status,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.tax.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_tax(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.tax.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Tax '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_tax(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.tax.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Tax '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
