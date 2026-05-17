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
async fn update_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.article.write(|store| -> Result<_, ApiError> {
        let entry = store
            .articles
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Article '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_articles))
        .route("/", post(add_article))
        .route("/{id}", get(get_article))
        .route("/{id}", delete(delete_article).patch(update_article))
}

#[derive(Debug, Deserialize)]
pub struct AddArticleRequest {
    pub name: String,
    pub title: String,
    pub url: String,
    pub source: String,
    pub status: String,
    pub notes: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub read_at: Option<String>,
}

async fn list_articles(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.article.read(|store| {
        let entries: Vec<_> = store.articles.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_article(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddArticleRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let title = req.title;
    let url = req.url;
    let source = req.source;
    let status: i_rs_article::models::ReadStatus =
        serde_json::from_value(serde_json::json!(req.status))
            .map_err(|e| ApiError::BadRequest(format!("Invalid status: {e}")))?;
    let notes = req.notes.unwrap_or_default();
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let read_at = req
        .read_at
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .transpose()?;
    let now = chrono::Utc::now();
    let entry = i_rs_article::models::Article {
        name,
        title,
        url,
        source,
        status,
        notes,
        tags,
        remark,
        created_at: now,
        updated_at: now,
        read_at,
    };
    state.article.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.article.read(|store| {
        store
            .articles
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Article '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.article.write(|store| {
        if store.articles.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Article '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
