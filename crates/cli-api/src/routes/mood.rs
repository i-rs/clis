use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::AppState;
use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};

async fn update_mood(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.mood.write(|store| -> Result<_, ApiError> {
        let key = store
            .entries
            .iter()
            .find(|(k, _)| k.starts_with(&id))
            .map(|(k, _)| k.clone())
            .ok_or_else(|| ApiError::NotFound(format!("Mood '{id}' not found")))?;
        let entry = store
            .entries
            .get_mut(&key)
            .ok_or_else(|| ApiError::NotFound(format!("Mood '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_moods))
        .route("/", post(add_mood))
        .route("/{id}", get(get_mood))
        .route("/{id}", delete(delete_mood).patch(update_mood))
        .route("/stats", get(mood_stats))
}

#[derive(Debug, Deserialize)]
pub struct AddMoodRequest {
    pub mood: String,
    pub tag: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub days: Option<i32>,
}

async fn list_moods(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let days = params.days.map(|d| d as usize);
    let records = state
        .mood
        .read(|store| i_rs_mood::service::list_moods(store, days).map_err(ApiError::from))?;
    Ok(ok_json_list(records))
}

async fn add_mood(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddMoodRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = req.date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let mood = req.mood;
    let tags = req.tag.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let record = state.mood.write(|store| {
        i_rs_mood::service::add_mood(&mut *store, date, mood, tags, remark)
            .map_err(ApiError::from)
    })?;
    Ok(ok_json(record))
}

async fn get_mood(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let record = state
        .mood
        .read(|store| i_rs_mood::service::get_mood(store, &id).map_err(ApiError::from))?;
    Ok(ok_json(record))
}

async fn delete_mood(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .mood
        .write(|store| i_rs_mood::service::delete_mood(store, id).map_err(ApiError::from))?;
    Ok(ok_json_message())
}

async fn mood_stats(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let stats = state
        .mood
        .read(|store| i_rs_mood::service::mood_stats(store).map_err(ApiError::from))?;
    match stats {
        Some((min, max, avg)) => Ok(ok_json(serde_json::json!({
            "best": min.label(),
            "worst": max.label(),
            "average": format!("{:.1}/7", avg),
        }))),
        None => Ok(ok_json(serde_json::json!({ "message": "No mood records" }))),
    }
}
