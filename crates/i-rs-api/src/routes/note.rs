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
        .route("/", get(list_notes))
        .route("/", post(add_note))
        .route("/:name", get(get_note))
        .route("/:name", delete(delete_note))
}

#[derive(Debug, Deserialize)]
pub struct AddNoteRequest {
    pub name: String,
    pub content: String,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
    pub search: Option<String>,
}

async fn list_notes(
    Query(params): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["list".to_string()];
    if let Some(ref tag) = params.tag {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }
    if let Some(ref search) = params.search {
        args.push("--search".to_string());
        args.push(search.clone());
    }

    run_cli("i-rs-note", args).await
}

async fn add_note(
    Json(req): Json<AddNoteRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["add".to_string(), req.name.clone()];
    args.push("--content".to_string());
    args.push(req.content.clone());
    for tag in req.tag.iter().flatten() {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }

    run_cli("i-rs-note", args).await
}

async fn get_note(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-note", vec!["get".to_string(), name]).await
}

async fn delete_note(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-note", vec!["delete".to_string(), name]).await
}
