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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_notes))
        .route("/", post(add_note))
        .route("/:name", get(get_note))
        .route("/:name", delete(delete_note))
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
    let records: Vec<i_rs_note::models::Note> = state.note.read(|store| {
        if let Some(ref tag) = params.tag {
            store
                .notes
                .values()
                .filter(|n| n.tags.contains(tag))
                .cloned()
                .collect()
        } else {
            store.notes.values().cloned().collect()
        }
    });
    Ok(ok_json_list(records))
}

async fn add_note(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddNoteRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let content = vec![req.content];
    let tags = req.tag.unwrap_or_default();

    let exists = state.note.read(|store| store.notes.contains_key(&name));
    if exists {
        return Err(ApiError::Conflict(format!("Note '{name}' already exists")));
    }

    let now = chrono::Utc::now();
    let note = state.note.write(|store| {
        let note = i_rs_note::models::Note {
            name: name.clone(),
            title: None,
            tags,
            content,
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        store.add_entry(note.clone());
        note
    });
    Ok(ok_json(note))
}

async fn get_note(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let note = state
        .note
        .read(|store| store.notes.get(&name).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("Note '{name}' not found")))?;
    Ok(ok_json(note))
}

async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.note.write(|store| {
        store
            .notes
            .remove(&name)
            .ok_or_else(|| anyhow::anyhow!("Note '{name}' not found"))
    })?;
    Ok(ok_json_message())
}
