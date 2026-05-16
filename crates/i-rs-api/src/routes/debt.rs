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
async fn update_debt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.debt.write(|store| -> Result<_, ApiError> {
        let entry = store
            .debts
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Debt '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_debts))
        .route("/", post(add_debt))
        .route("/{id}", get(get_debt))
        .route("/{id}", delete(delete_debt).patch(update_debt))
}

#[derive(Debug, Deserialize)]
pub struct AddDebtRequest {
    pub name: String,
    pub debt_type: String,
    pub total_amount: f64,
    pub interest_rate: Option<f64>,
    pub due_date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_debts(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.debt.read(|store| {
        let entries: Vec<_> = store.debts.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_debt(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddDebtRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let debt_type: i_rs_debt::models::DebtType = req
        .debt_type
        .parse()
        .map_err(|e| ApiError::BadRequest(format!("Invalid debt type: {e}")))?;
    let total_amount = req.total_amount;
    let interest_rate = req.interest_rate;
    let due_date = req
        .due_date
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|_| {
                    ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string())
                })
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .transpose()?;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let mut entry = i_rs_debt::models::Debt::new(name, debt_type, total_amount);
    entry.interest_rate = interest_rate;
    entry.due_date = due_date;
    entry.tags = tags;
    entry.remark = remark;
    state.debt.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_debt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.debt.read(|store| {
        store
            .debts
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Debt '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_debt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.debt.write(|store| {
        if store.debts.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Debt '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
