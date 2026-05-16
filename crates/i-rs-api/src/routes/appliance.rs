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
async fn update_appliance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.appliance.write(|store| -> Result<_, ApiError> {
        let entry = store
            .appliances
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Appliance '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_appliances))
        .route("/", post(add_appliance))
        .route("/{id}", get(get_appliance))
        .route("/{id}", delete(delete_appliance).patch(update_appliance))
}

#[derive(Debug, Deserialize)]
pub struct AddApplianceRequest {
    pub name: String,
    pub brand: String,
    pub model: String,
    pub purchase_date: String,
    pub lifespan_years: u32,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub maintenance_records: Option<serde_json::Value>,
}

async fn list_appliances(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.appliance.read(|store| {
        let entries: Vec<_> = store.appliances.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_appliance(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddApplianceRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let brand = req.brand;
    let model = req.model;
    let parsed_purchase_date = chrono::DateTime::parse_from_rfc3339(&req.purchase_date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let purchase_date = parsed_purchase_date.with_timezone(&chrono::Utc);
    let lifespan_years = req.lifespan_years;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let maintenance_records: Vec<i_rs_appliance::models::MaintenanceRecord> = req
        .maintenance_records
        .map(|v| {
            serde_json::from_value(v)
                .map_err(|e| ApiError::BadRequest(format!("Invalid maintenance_records: {e}")))
        })
        .transpose()?
        .unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_appliance::models::Appliance {
        id: id.clone(),
        name,
        brand,
        model,
        purchase_date,
        lifespan_years,
        tags,
        remark,
        maintenance_records,
        created_at: now,
        updated_at: now,
    };
    state.appliance.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_appliance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.appliance.read(|store| {
        store
            .appliances
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Appliance '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_appliance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.appliance.write(|store| {
        if store.appliances.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Appliance '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
