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
async fn update_quote(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.quote.write(|store| -> Result<_, ApiError> {
        let entry = store.quotes.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Quote '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_quotes))
        .route("/", post(add_quote))
        .route("/{id}", get(get_quote))
        .route("/{id}", delete(delete_quote).put(update_quote))
}

#[derive(Debug, Deserialize)]
pub struct AddQuoteRequest {
    pub content: String,
    pub author: Option<String>,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_quotes(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.quote.read(|store| {
        let entries: Vec<_> = store.quotes.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_quote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddQuoteRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let content = req.content;
    let author = req.author;
    let source = req.source;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_quote::models::Quote {
        id: id.clone(),
        content,
        author,
        source,
        tags,
        remark,
        created_at: now,
    };
    state.quote.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_quote(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.quote.read(|store| {
        store.quotes.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Quote '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_quote(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.quote.write(|store| {
        if store.quotes.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Quote '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
