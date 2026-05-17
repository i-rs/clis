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
async fn update_spark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.spark.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Spark '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_sparks))
        .route("/", post(add_spark))
        .route("/{id}", get(get_spark))
        .route("/{id}", delete(delete_spark).patch(update_spark))
}

#[derive(Debug, Deserialize)]
pub struct AddSparkRequest {
    pub content: String,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_sparks(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.spark.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_spark(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSparkRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let content = req.content;
    let source = req.source;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let entry = i_rs_spark::models::SparkEntry::new(content, source, tags, remark);
    state.spark.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_spark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.spark.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Spark '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_spark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.spark.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Spark '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
