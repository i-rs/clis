use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};
use crate::AppState;
async fn update_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.note.write(|store| -> Result<_, ApiError> {
        let entry = store.notes.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Note '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_notes))
        .route("/", post(add_note))
        .route("/{name}", get(get_note))
        .route("/{name}", delete(delete_note).put(update_note))
}

#[derive(Debug, Deserialize)]
pub struct AddNoteRequest {
    pub name: String,
    pub content: String,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
}

async fn list_notes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.note.read(|store| {
        i_rs_note::service::list_notes(store, params.tag.clone()).map_err(ApiError::from)
    })?;
    Ok(ok_json_list(records))
}

async fn add_note(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddNoteRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let content = vec![req.content];
    let tags = req.tag.unwrap_or_default();
    let note = state.note.write(|store| {
        i_rs_note::service::add_note(store, name, None, tags, content).map_err(ApiError::from)
    })?;
    Ok(ok_json(note))
}

async fn get_note(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let note = state.note.read(|store| {
        i_rs_note::service::get_note(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(note))
}

async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.note.write(|store| {
        i_rs_note::service::delete_note(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json_message())
}
