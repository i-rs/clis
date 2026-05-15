use axum::{
    Router,
    routing::{get, post},
    extract::Path,
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::run_cli;

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_habits))
        .route("/", post(add_habit))
        .route("/:name", get(get_habit))
        .route("/:name/checkin", post(checkin_habit))
        .route("/:name/stats", get(habit_stats))
}

#[derive(Debug, Deserialize)]
pub struct AddHabitRequest {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub tag: Option<Vec<String>>,
}

async fn list_habits() -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-habit", vec!["list".to_string()]).await
}

async fn add_habit(
    Json(req): Json<AddHabitRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["add".to_string(), req.name.clone()];
    if let Some(ref title) = req.title {
        args.push("--title".to_string());
        args.push(title.clone());
    }
    if let Some(ref description) = req.description {
        args.push("--description".to_string());
        args.push(description.clone());
    }
    if let Some(ref frequency) = req.frequency {
        args.push("--frequency".to_string());
        args.push(frequency.clone());
    }
    for tag in req.tag.iter().flatten() {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }

    run_cli("i-rs-habit", args).await
}

async fn get_habit(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-habit", vec!["get".to_string(), name]).await
}

async fn checkin_habit(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-habit", vec!["checkin".to_string(), name]).await
}

async fn habit_stats(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-habit", vec!["stats".to_string(), name]).await
}
