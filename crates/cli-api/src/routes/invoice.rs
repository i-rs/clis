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
async fn update_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.invoice.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Invoice '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_invoices))
        .route("/", post(add_invoice))
        .route("/{id}", get(get_invoice))
        .route("/{id}", delete(delete_invoice).patch(update_invoice))
}

#[derive(Debug, Deserialize)]
pub struct AddInvoiceRequest {
    pub name: String,
    pub amount: f64,
    pub date: String,
    pub invoice_type: String,
    pub reimbursed: bool,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_invoices(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.invoice.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_invoice(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddInvoiceRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let amount = req.amount;
    let parsed_date = chrono::DateTime::parse_from_rfc3339(&req.date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let date = parsed_date.with_timezone(&chrono::Utc);
    let invoice_type: i_rs_invoice::models::InvoiceType = req
        .invoice_type
        .parse()
        .map_err(|e| ApiError::BadRequest(format!("Invalid invoice_type: {e}")))?;
    let reimbursed = req.reimbursed;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_invoice::models::Invoice {
        id: id.clone(),
        name,
        amount,
        date,
        invoice_type,
        reimbursed,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.invoice.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.invoice.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Invoice '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.invoice.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Invoice '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
