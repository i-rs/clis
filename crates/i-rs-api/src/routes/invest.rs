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
async fn update_invest(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.invest.write(|store| -> Result<_, ApiError> {
        let entry = store.investments.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Invest '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_invests))
        .route("/", post(add_invest))
        .route("/{id}", get(get_invest))
        .route("/{id}", delete(delete_invest).patch(update_invest))
}

#[derive(Debug, Deserialize)]
pub struct AddInvestmentRequest {
    pub name: String,
    pub symbol: String,
    pub asset_type: String,
    pub quantity: f64,
    pub buy_price: f64,
    pub buy_date: String,
    pub current_price: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_invests(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.invest.read(|store| {
        let entries: Vec<_> = store.investments.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_invest(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddInvestmentRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let symbol = req.symbol;
    let asset_type: i_rs_invest::models::AssetType = req.asset_type.parse()
        .map_err(|e| ApiError::BadRequest(format!("Invalid asset_type: {e}")))?;
    let quantity = req.quantity;
    let buy_price = req.buy_price;
    let parsed_buy_date = chrono::DateTime::parse_from_rfc3339(&req.buy_date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let buy_date = parsed_buy_date.with_timezone(&chrono::Utc);
    let current_price = req.current_price;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_invest::models::Investment {
        name,
        symbol,
        asset_type,
        quantity,
        buy_price,
        buy_date,
        current_price,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.invest.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_invest(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.invest.read(|store| {
        store.investments.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Invest '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_invest(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.invest.write(|store| {
        if store.investments.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Invest '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
