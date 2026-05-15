use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, Query},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::{call_service, call_service_unit};

pub fn router() -> Router {
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
    Query(params): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let days = params.days.map(|d| d as usize);
    call_service(move || i_rs_mood::service::list_moods(days)).await
}

async fn add_mood(
    Json(req): Json<AddMoodRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mood = req.mood;
    let content = req.note.map(|n| vec![n]).unwrap_or_default();
    let tags = req.tag.unwrap_or_default();
    call_service(move || i_rs_mood::service::add_mood(date, mood, tags, content)).await
}

async fn get_mood(
    Path(date): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_mood::service::get_mood(&date)).await
}

async fn delete_mood(
    Path(date): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service_unit(move || i_rs_mood::service::delete_mood(date)).await
}

async fn mood_stats() -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(|| -> anyhow::Result<serde_json::Value> {
        let stats = i_rs_mood::service::mood_stats()?;
        match stats {
            Some((min, max, avg)) => Ok(serde_json::json!({
                "best": min.label(),
                "worst": max.label(),
                "average": format!("{:.1}/5", avg),
            })),
            None => Ok(serde_json::json!({ "message": "No mood records" })),
        }
    }).await
}
