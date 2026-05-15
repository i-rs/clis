use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};
use crate::AppState;

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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_todos))
        .route("/", post(add_todo))
        .route("/:name", get(get_todo))
        .route("/:name", put(update_todo))
        .route("/:name/done", post(done_todo))
        .route("/:name", delete(delete_todo))
}

async fn list_todos(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let pending = params.pending.unwrap_or(false);
    let done = params.done.unwrap_or(false);
    let records = state.todo.read(|store| {
        i_rs_todo::service::list_todos(store, pending, done, params.tag.clone()).map_err(ApiError::from)
    })?;
    Ok(ok_json_list(records))
}

async fn add_todo(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddTodoRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let title = req.title;
    let priority = req.priority;
    let tag = req.tag.unwrap_or_default();
    let content = req.content.unwrap_or_default();
    let record = state.todo.write(|store| {
        i_rs_todo::service::add_todo(store, name, title, priority, tag, content).map_err(ApiError::from)
    })?;
    Ok(ok_json(record))
}

async fn get_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let todo = state.todo.read(|store| {
        i_rs_todo::service::get_todo(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(todo))
}

async fn update_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<UpdateTodoRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let todo = state.todo.write(|store| {
        i_rs_todo::service::update_todo(store, name, req.title, req.priority, req.tag, req.content).map_err(ApiError::from)
    })?;
    Ok(ok_json(todo))
}

async fn done_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let todo = state.todo.write(|store| {
        i_rs_todo::service::toggle_todo_done(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(todo))
}

async fn delete_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.todo.write(|store| {
        i_rs_todo::service::delete_todo(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json_message())
}
