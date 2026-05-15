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
async fn update_bookmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.bookmark.write(|store| -> Result<_, ApiError> {
        let entry = store.bookmarks.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Bookmark '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_bookmarks))
        .route("/", post(add_bookmark))
        .route("/{name}", get(get_bookmark))
        .route("/{name}", delete(delete_bookmark).put(update_bookmark))
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
    let records = state.bookmark.read(|store| {
        i_rs_bookmark::service::list_bookmarks(store, params.tag.clone()).map_err(ApiError::from)
    })?;
    Ok(ok_json_list(records))
}

async fn add_bookmark(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddBookmarkRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let url = req.url;
    let tags = req.tag.unwrap_or_default();
    let bookmark = state.bookmark.write(|store| {
        i_rs_bookmark::service::add_bookmark(store, name, url, None, None, tags, Vec::new()).map_err(ApiError::from)
    })?;
    Ok(ok_json(bookmark))
}

async fn get_bookmark(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let bookmark = state.bookmark.read(|store| {
        i_rs_bookmark::service::get_bookmark(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(bookmark))
}

async fn delete_bookmark(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.bookmark.write(|store| {
        i_rs_bookmark::service::delete_bookmark(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json_message())
}
