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
async fn update_bestby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.bestby.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Bestby '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_bestbys))
        .route("/", post(add_bestby))
        .route("/{id}", get(get_bestby))
        .route("/{id}", delete(delete_bestby).patch(update_bestby))
}

#[derive(Debug, Deserialize)]
pub struct AddEntityRequest {
    pub name: String,
    pub purchase_date: String,
    pub cycle_days: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_bestbys(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.bestby.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_bestby(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddEntityRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let parsed_purchase_date = chrono::DateTime::parse_from_rfc3339(&req.purchase_date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let purchase_date = parsed_purchase_date.with_timezone(&chrono::Utc);
    let cycle_days = req.cycle_days;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_bestby::models::Entity {
        name,
        purchase_date,
        cycle_days,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.bestby.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_bestby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.bestby.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Bestby '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_bestby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.bestby.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Bestby '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
