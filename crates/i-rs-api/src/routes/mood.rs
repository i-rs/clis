use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, Query},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::run_cli;

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
    pub tag: Option<String>,
}

async fn list_moods(
    Query(params): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["list".to_string()];
    if let Some(days) = params.days {
        args.push("--days".to_string());
        args.push(days.to_string());
    }
    if let Some(ref tag) = params.tag {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }

    run_cli("i-rs-mood", args).await
}

async fn add_mood(
    Json(req): Json<AddMoodRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["add".to_string(), req.mood.clone()];
    if let Some(ref note) = req.note {
        args.push("--note".to_string());
        args.push(note.clone());
    }
    for tag in req.tag.iter().flatten() {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }

    run_cli("i-rs-mood", args).await
}

async fn get_mood(
    Path(date): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-mood", vec!["get".to_string(), date]).await
}

async fn delete_mood(
    Path(date): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-mood", vec!["delete".to_string(), date]).await
}

async fn mood_stats() -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-mood", vec!["stats".to_string()]).await
}
