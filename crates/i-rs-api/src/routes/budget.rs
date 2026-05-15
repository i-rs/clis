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
        .route("/", get(list_budgets))
        .route("/", post(add_budget))
        .route("/{id}", get(get_budget))
        .route("/{id}", delete(delete_budget))
}

#[derive(Debug, Deserialize)]
pub struct AddBudgetRequest {
    pub category: String,
    pub amount: f64,
    pub period: String,
}

async fn list_budgets(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.budget.read(|store| {
        let entries: Vec<_> = store.budgets.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_budget(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddBudgetRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let category = req.category;
    let amount = req.amount;
    let period: i_rs_budget::models::BudgetPeriod =
        serde_json::from_value(serde_json::json!(req.period))
            .map_err(|_| ApiError::BadRequest(format!("Invalid period: {}", req.period)))?;
    let entry = i_rs_budget::models::Budget::new(category, amount, period);
    state.budget.write(|store| {
        let b = entry.clone();
        store.budgets.insert(b.category.clone(), b);
    });
    Ok(ok_json(entry))
}

async fn get_budget(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.budget.read(|store| {
        store.budgets.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Budget '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_budget(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.budget.write(|store| {
        if store.budgets.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Budget '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
