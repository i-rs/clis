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
async fn update_podcast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.podcast.write(|store| -> Result<_, ApiError> {
        let entry = store.podcasts.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Podcast '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_podcasts))
        .route("/", post(add_podcast))
        .route("/{id}", get(get_podcast))
        .route("/{id}", delete(delete_podcast).patch(update_podcast))
}

#[derive(Debug, Deserialize)]
pub struct AddPodcastRequest {
    pub name: String,
    pub author: Option<String>,
    pub duration_secs: Option<i64>,
    pub current_position_secs: Option<i64>,
    pub status: String,
    pub notes: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_podcasts(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.podcast.read(|store| {
        let entries: Vec<_> = store.podcasts.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_podcast(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddPodcastRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let author = req.author;
    let duration_secs = req.duration_secs;
    let current_position_secs = req.current_position_secs;
    let status: i_rs_podcast::models::PodcastStatus = serde_json::from_value(serde_json::json!(req.status))
        .map_err(|e| ApiError::BadRequest(format!("Invalid status: {e}")))?;
    let notes = req.notes.unwrap_or_default();
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_podcast::models::Podcast {
        name,
        author,
        duration_secs,
        current_position_secs,
        status,
        notes,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.podcast.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_podcast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.podcast.read(|store| {
        store.podcasts.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Podcast '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_podcast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.podcast.write(|store| {
        if store.podcasts.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Podcast '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
