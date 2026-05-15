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
        .route("/", get(list_keys))
        .route("/:name", post(add_key))
        .route("/:name", get(get_key))
        .route("/:name", delete(delete_key))
}

#[derive(Debug, Deserialize)]
pub struct AddKeyRequest {
    pub value: String,
    pub remark: Option<String>,
}

async fn list_keys(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.keys.read(|store| {
        i_rs_keys::service::list_keys(store, None).map_err(ApiError::from)
    })?;
    Ok(ok_json_list(records))
}

async fn add_key(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<AddKeyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let value = req.value;
    let remark = req.remark.unwrap_or_default();
    let entry = state.keys.write(|store| {
        i_rs_keys::service::add_key(store, name, "api_key".to_string(), value, Vec::new(), vec![remark]).map_err(ApiError::from)
    })?;
    Ok(ok_json(entry))
}

async fn get_key(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.keys.read(|store| {
        i_rs_keys::service::get_key(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(entry))
}

async fn delete_key(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.keys.write(|store| {
        i_rs_keys::service::delete_key(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json_message())
}
