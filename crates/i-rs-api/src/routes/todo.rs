use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{Path, Query},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::{call_service, call_service_unit};

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
    let pending = params.pending.unwrap_or(false);
    let done = params.done.unwrap_or(false);
    let tag = params.tag;
    call_service(move || i_rs_todo::service::list_todos(pending, done, tag)).await
}

async fn add_todo(
    Json(req): Json<AddTodoRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let name = req.name;
    let title = req.title;
    let priority = req.priority;
    let tag = req.tag.unwrap_or_default();
    let content = req.content.unwrap_or_default();
    call_service(move || i_rs_todo::service::add_todo(name, title, priority, tag, content)).await
}

async fn get_todo(Path(name): Path<String>) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_todo::service::get_todo(&name)).await
}

async fn update_todo(
    Path(name): Path<String>,
    Json(req): Json<UpdateTodoRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let name = name;
    let title = req.title;
    let priority = req.priority;
    let tag = req.tag;
    let content = req.content;
    call_service(move || i_rs_todo::service::update_todo(name, title, priority, tag, content)).await
}

async fn done_todo(Path(name): Path<String>) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_todo::service::toggle_todo_done(&name)).await
}

async fn delete_todo(Path(name): Path<String>) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service_unit(move || i_rs_todo::service::delete_todo(&name)).await
}
