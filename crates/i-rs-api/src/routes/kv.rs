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
        .route("/", get(list_kv))
        .route("/:key", get(get_kv))
        .route("/:key", post(set_kv))
        .route("/:key", delete(delete_kv))
}

#[derive(Debug, Deserialize)]
pub struct SetKvRequest {
    pub value: String,
}

async fn list_kv(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.kv.read(|store| {
        i_rs_kv::service::list_kv(store, None).map_err(ApiError::from)
    })?;
    Ok(ok_json_list(records))
}

async fn set_kv(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<SetKvRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let value = req.value;
    let entry = state.kv.write(|store| {
        i_rs_kv::service::add_kv(store, key, value, Vec::new(), Vec::new()).map_err(ApiError::from)
    })?;
    Ok(ok_json(entry))
}

async fn get_kv(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.kv.read(|store| {
        i_rs_kv::service::get_kv(store, &key).map_err(ApiError::from)
    })?;
    Ok(ok_json(entry))
}

async fn delete_kv(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.kv.write(|store| {
        i_rs_kv::service::delete_kv(store, &key).map_err(ApiError::from)
    })?;
    Ok(ok_json_message())
}
