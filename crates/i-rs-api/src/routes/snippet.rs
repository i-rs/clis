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
async fn update_snippet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.snippet.write(|store| -> Result<_, ApiError> {
        let entry = store.snippets.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Snippet '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_snippets))
        .route("/", post(add_snippet))
        .route("/{id}", get(get_snippet))
        .route("/{id}", delete(delete_snippet).put(update_snippet))
}

#[derive(Debug, Deserialize)]
pub struct AddSnippetRequest {
    pub name: String,
    pub language: String,
    pub code: Option<Vec<String>>,
    pub description: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_snippets(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.snippet.read(|store| {
        let entries: Vec<_> = store.snippets.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_snippet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSnippetRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let language = req.language;
    let code = req.code.unwrap_or_default();
    let description = req.description.unwrap_or_default();
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_snippet::models::Snippet {
        id: id.clone(),
        name,
        language,
        code,
        description,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.snippet.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_snippet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.snippet.read(|store| {
        store.snippets.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Snippet '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_snippet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.snippet.write(|store| {
        if store.snippets.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Snippet '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
