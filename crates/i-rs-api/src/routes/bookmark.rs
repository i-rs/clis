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
        .route("/", get(list_bookmarks))
        .route("/", post(add_bookmark))
        .route("/:name", get(get_bookmark))
        .route("/:name", delete(delete_bookmark))
}

#[derive(Debug, Deserialize)]
pub struct AddBookmarkRequest {
    pub name: String,
    pub url: String,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
}

async fn list_bookmarks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let records: Vec<i_rs_bookmark::models::Bookmark> = state.bookmark.read(|store| {
        if let Some(ref tag) = params.tag {
            store
                .bookmarks
                .values()
                .filter(|b| b.tags.contains(tag))
                .cloned()
                .collect()
        } else {
            store.bookmarks.values().cloned().collect()
        }
    });
    Ok(ok_json_list(records))
}

async fn add_bookmark(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddBookmarkRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let url = req.url;
    let tags = req.tag.unwrap_or_default();

    let exists = state.bookmark.read(|store| store.bookmarks.contains_key(&name));
    if exists {
        return Err(ApiError::Conflict(format!("Bookmark '{name}' already exists")));
    }

    let now = chrono::Utc::now();
    let bookmark = state.bookmark.write(|store| {
        let bookmark = i_rs_bookmark::models::Bookmark {
            name: name.clone(),
            url,
            account: None,
            password: None,
            tags,
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        store.add_entry(bookmark.clone());
        bookmark
    });
    Ok(ok_json(bookmark))
}

async fn get_bookmark(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let bookmark = state
        .bookmark
        .read(|store| store.bookmarks.get(&name).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("Bookmark '{name}' not found")))?;
    Ok(ok_json(bookmark))
}

async fn delete_bookmark(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.bookmark.write(|store| {
        store
            .bookmarks
            .remove(&name)
            .ok_or_else(|| anyhow::anyhow!("Bookmark '{name}' not found"))
    })?;
    Ok(ok_json_message())
}
