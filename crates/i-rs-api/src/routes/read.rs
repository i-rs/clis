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
async fn update_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.read.write(|store| -> Result<_, ApiError> {
        let entry = store
            .books
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Read '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_reads))
        .route("/", post(add_read))
        .route("/{id}", get(get_read))
        .route("/{id}", delete(delete_read).patch(update_read))
}

#[derive(Debug, Deserialize)]
pub struct AddReadRequest {
    pub name: String,
    pub author: String,
    pub total_pages: u32,
}

async fn list_reads(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.read.read(|store| {
        let entries: Vec<_> = store.books.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_read(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddReadRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let author = req.author;
    let total_pages = req.total_pages;
    let entry = i_rs_read::models::Book::new(name, author, total_pages);
    state.read.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.read.read(|store| {
        store
            .books
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Read '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.read.write(|store| {
        if store.books.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Read '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
