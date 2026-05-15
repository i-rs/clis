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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_reminds))
        .route("/", post(add_remind))
        .route("/{id}", get(get_remind))
        .route("/{id}", delete(delete_remind))
}

#[derive(Debug, Deserialize)]
pub struct AddRemindRequest {
    pub name: String,
    pub event_date: String,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub content: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub is_done: bool,
}

async fn list_reminds(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.remind.read(|store| {
        let entries: Vec<_> = store.reminds.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_remind(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddRemindRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let parsed_event_date = chrono::DateTime::parse_from_rfc3339(&req.event_date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let event_date = parsed_event_date.with_timezone(&chrono::Utc);
    let title = req.title;
    let tags = req.tags.unwrap_or_default();
    let content = req.content.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let is_done = req.is_done;
    let now = chrono::Utc::now();
    let entry = i_rs_remind::models::Remind {
        name,
        event_date,
        title,
        tags,
        content,
        remark,
        is_done,
        created_at: now,
        updated_at: now,
    };
    state.remind.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_remind(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.remind.read(|store| {
        store.reminds.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Remind '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_remind(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.remind.write(|store| {
        if store.reminds.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Remind '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
