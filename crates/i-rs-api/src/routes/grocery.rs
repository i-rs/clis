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
async fn update_grocery(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.grocery.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Grocery '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_grocerys))
        .route("/", post(add_grocery))
        .route("/{id}", get(get_grocery))
        .route("/{id}", delete(delete_grocery).put(update_grocery))
}

#[derive(Debug, Deserialize)]
pub struct AddGroceryItemRequest {
    pub name: String,
    pub quantity: i32,
    pub unit: String,
    pub purchased: bool,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_grocerys(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.grocery.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_grocery(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddGroceryItemRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let quantity = req.quantity;
    let unit = req.unit;
    let purchased = req.purchased;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_grocery::models::GroceryItem {
        name,
        quantity,
        unit,
        purchased,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.grocery.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_grocery(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.grocery.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Grocery '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_grocery(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.grocery.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Grocery '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
