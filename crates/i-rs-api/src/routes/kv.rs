use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
};
use serde::Deserialize;

use crate::AppState;
use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct CopyRequest {
    pub dst: String,
}

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub new: String,
}

async fn update_kv(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.kv.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Kv '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_kv))
        .route("/search", get(search_kv_handler))
        .route("/stats", get(kv_stats_handler))
        .route("/{key}", get(get_kv))
        .route("/{key}", post(set_kv))
        .route("/{key}", delete(delete_kv).patch(update_kv))
        .route("/{key}/copy", post(copy_kv_handler))
        .route("/{key}/rename", patch(rename_kv_handler))
}

#[derive(Debug, Deserialize)]
pub struct SetKvRequest {
    pub value: String,
}

async fn list_kv(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.kv.read(|store| {
        i_rs_kv::service::list_kv(store, params.tag, params.pattern.as_deref())
            .map_err(ApiError::from)
    })?;
    Ok(ok_json_list(records))
}

async fn search_kv_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state
        .kv
        .read(|store| i_rs_kv::service::search_kv(store, &params.q).map_err(ApiError::from))?;
    Ok(ok_json_list(records))
}

async fn kv_stats_handler(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let stats = state
        .kv
        .read(|store| i_rs_kv::service::stats_kv(store).map_err(ApiError::from))?;
    Ok(ok_json(stats))
}

async fn copy_kv_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<CopyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state
        .kv
        .write(|store| i_rs_kv::service::copy_kv(store, &key, req.dst).map_err(ApiError::from))?;
    Ok(ok_json(entry))
}

async fn rename_kv_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<RenameRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state
        .kv
        .write(|store| i_rs_kv::service::rename_kv(store, &key, req.new).map_err(ApiError::from))?;
    Ok(ok_json(entry))
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
    let entry = state
        .kv
        .read(|store| i_rs_kv::service::get_kv(store, &key).map_err(ApiError::from))?;
    Ok(ok_json(entry))
}

async fn delete_kv(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .kv
        .write(|store| i_rs_kv::service::delete_kv(store, &key).map_err(ApiError::from))?;
    Ok(ok_json_message())
}
