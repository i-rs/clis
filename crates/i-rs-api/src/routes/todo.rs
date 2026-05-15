use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{Path, Query},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::run_cli;

#[derive(Debug, Deserialize)]
pub struct AddTodoRequest {
    pub name: String,
    pub title: Option<String>,
    pub content: Option<Vec<String>>,
    pub priority: Option<String>,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub content: Option<Vec<String>>,
    pub priority: Option<String>,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub pending: Option<bool>,
    pub done: Option<bool>,
    pub tag: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_todos))
        .route("/", post(add_todo))
        .route("/:name", get(get_todo))
        .route("/:name", put(update_todo))
        .route("/:name/done", post(done_todo))
        .route("/:name", delete(delete_todo))
}

async fn list_todos(
    Query(params): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["list".to_string()];
    if params.pending.unwrap_or(false) {
        args.push("--pending".to_string());
    }
    if params.done.unwrap_or(false) {
        args.push("--done".to_string());
    }
    if let Some(ref tag) = params.tag {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }
    run_cli("i-rs-todo", args).await
}

async fn add_todo(
    Json(req): Json<AddTodoRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["add".to_string(), req.name.clone()];
    if let Some(ref title) = req.title {
        args.push("--title".to_string());
        args.push(title.clone());
    }
    if let Some(ref priority) = req.priority {
        args.push("-p".to_string());
        args.push(priority.clone());
    }
    for tag in req.tag.iter().flatten() {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }
    for content in req.content.iter().flatten() {
        args.push("--content".to_string());
        args.push(content.clone());
    }
    run_cli("i-rs-todo", args).await
}

async fn get_todo(Path(name): Path<String>) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-todo", vec!["get".to_string(), name]).await
}

async fn update_todo(
    Path(name): Path<String>,
    Json(req): Json<UpdateTodoRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["update".to_string(), name];
    if let Some(ref title) = req.title {
        args.push("--title".to_string());
        args.push(title.clone());
    }
    if let Some(ref priority) = req.priority {
        args.push("-p".to_string());
        args.push(priority.clone());
    }
    if let Some(ref tags) = req.tag {
        for tag in tags {
            args.push("--tag".to_string());
            args.push(tag.clone());
        }
    }
    if let Some(ref contents) = req.content {
        for content in contents {
            args.push("--content".to_string());
            args.push(content.clone());
        }
    }
    run_cli("i-rs-todo", args).await
}

async fn done_todo(Path(name): Path<String>) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-todo", vec!["done".to_string(), name]).await
}

async fn delete_todo(Path(name): Path<String>) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-todo", vec!["delete".to_string(), name]).await
}
