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
async fn update_vocab(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.vocab.write(|store| -> Result<_, ApiError> {
        let entry = store
            .words
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Vocab '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_vocabs))
        .route("/", post(add_vocab))
        .route("/{id}", get(get_vocab))
        .route("/{id}", delete(delete_vocab).patch(update_vocab))
}

#[derive(Debug, Deserialize)]
pub struct AddVocabWordRequest {
    pub word: String,
    pub definition: String,
    pub example: Option<Vec<String>>,
    pub status: String,
    pub review_count: u32,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_vocabs(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.vocab.read(|store| {
        let entries: Vec<_> = store.words.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_vocab(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddVocabWordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let word = req.word;
    let definition = req.definition;
    let example = req.example.unwrap_or_default();
    let status: i_rs_vocab::models::VocabStatus =
        serde_json::from_value(serde_json::json!(req.status))
            .map_err(|e| ApiError::BadRequest(format!("Invalid status: {e}")))?;
    let review_count = req.review_count;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_vocab::models::VocabWord {
        word,
        definition,
        example,
        status,
        review_count,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.vocab.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_vocab(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.vocab.read(|store| {
        store
            .words
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Vocab '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_vocab(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.vocab.write(|store| {
        if store.words.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Vocab '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
