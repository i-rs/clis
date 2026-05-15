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
use i_rs_core::parse_date;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_moods))
        .route("/", post(add_mood))
        .route("/:date", get(get_mood))
        .route("/:date", delete(delete_mood))
        .route("/stats", get(mood_stats))
}

#[derive(Debug, Deserialize)]
pub struct AddMoodRequest {
    pub mood: String,
    pub note: Option<String>,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub days: Option<i32>,
}

async fn list_moods(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let records: Vec<i_rs_mood::models::MoodRecord> = state.mood.read(|store| {
        if let Some(days) = params.days {
            store
                .get_recent_records(days as usize)
                .into_iter()
                .cloned()
                .collect()
        } else {
            store.get_all_records().into_iter().cloned().collect()
        }
    });
    Ok(ok_json_list(records))
}

async fn add_mood(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddMoodRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let parsed_date = parse_date(&date)?;
    let mood = req.mood;
    let content = req.note.map(|n| vec![n]).unwrap_or_default();
    let tags = req.tag.unwrap_or_default();

    // Check for duplicate before write
    let exists = state.mood.read(|store| store.records.contains_key(&parsed_date));
    if exists {
        return Err(ApiError::Conflict(format!("Mood record for {date} already exists")));
    }

    let mood_val = parse_mood(&mood);
    let now = chrono::Utc::now();

    let record = state.mood.write(|store| {
        let record = i_rs_mood::models::MoodRecord {
            date: parsed_date,
            mood: mood_val,
            tags,
            content,
            remark: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        store.add_entry(record.clone());
        record
    });
    Ok(ok_json(record))
}

async fn get_mood(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let parsed_date = parse_date(&date)?;
    let record = state
        .mood
        .read(|store| store.get_entry(&parsed_date).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("No mood record found for {date}")))?;
    Ok(ok_json(record))
}

async fn delete_mood(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let parsed_date = parse_date(&date)?;
    state.mood.write(|store| {
        store
            .remove_entry(&parsed_date)
            .ok_or_else(|| anyhow::anyhow!("No mood record found for {date}"))
    })?;
    Ok(ok_json_message())
}

async fn mood_stats(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let stats = state.mood.read(|store| store.mood_stats());
    match stats {
        Some((min, max, avg)) => Ok(ok_json(serde_json::json!({
            "best": min.label(),
            "worst": max.label(),
            "average": format!("{:.1}/5", avg),
        }))),
        None => Ok(ok_json(serde_json::json!({ "message": "No mood records" }))),
    }
}

fn parse_mood(s: &str) -> i_rs_mood::models::Mood {
    match s.to_lowercase().as_str() {
        "5" | "great" | "😊" => i_rs_mood::models::Mood::Great,
        "4" | "good" | "🙂" => i_rs_mood::models::Mood::Good,
        "3" | "okay" | "😐" => i_rs_mood::models::Mood::Okay,
        "2" | "bad" | "😔" => i_rs_mood::models::Mood::Bad,
        "1" | "terrible" | "😢" => i_rs_mood::models::Mood::Terrible,
        _ => i_rs_mood::models::Mood::Okay,
    }
}
