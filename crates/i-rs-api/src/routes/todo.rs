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
    let records: Vec<i_rs_todo::models::Todo> = state.todo.read(|store| {
        let pending = params.pending.unwrap_or(false);
        let done = params.done.unwrap_or(false);
        let todos: Vec<&i_rs_todo::models::Todo> = if pending && !done {
            store.get_pending_todos()
        } else if done && !pending {
            store.get_done_todos()
        } else {
            store.get_all_todos()
        };
        let filtered = if let Some(ref tag) = params.tag {
            todos.into_iter().filter(|t| t.tags.contains(tag)).collect()
        } else {
            todos
        };
        filtered.into_iter().cloned().collect()
    });
    Ok(ok_json_list(records))
}

async fn add_todo(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddTodoRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let title = req.title;
    let priority = req.priority.and_then(|p| i_rs_todo::models::Priority::from_str(&p)).unwrap_or_default();
    let tag = req.tag.unwrap_or_default();
    let content = req.content.unwrap_or_default();

    let exists = state.todo.read(|store| store.todos.contains_key(&name));
    if exists {
        return Err(ApiError::Conflict(format!("Todo '{name}' already exists")));
    }

    let now = chrono::Utc::now();
    let todo = state.todo.write(|store| {
        let todo = i_rs_todo::models::Todo {
            name: name.clone(),
            title,
            priority,
            tags: tag,
            content,
            is_done: false,
            created_at: now,
            updated_at: now,
        };
        store.add_entry(todo.clone());
        todo
    });
    Ok(ok_json(todo))
}

async fn get_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let todo = state
        .todo
        .read(|store| store.todos.get(&name).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("Todo '{name}' not found")))?;
    Ok(ok_json(todo))
}

async fn update_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<UpdateTodoRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let todo = state.todo.write(|store| {
        let todo = store
            .todos
            .get_mut(&name)
            .ok_or_else(|| anyhow::anyhow!("Todo '{name}' not found"))?;
        if let Some(title) = req.title {
            todo.title = Some(title);
        }
        if let Some(priority_str) = req.priority {
            if let Some(p) = i_rs_todo::models::Priority::from_str(&priority_str) {
                todo.priority = p;
            }
        }
        if let Some(tag) = req.tag {
            todo.tags = tag;
        }
        if let Some(content) = req.content {
            todo.content = content;
        }
        todo.updated_at = chrono::Utc::now();
        Ok::<_, anyhow::Error>(todo.clone())
    })?;
    Ok(ok_json(todo))
}

async fn done_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let todo = state.todo.write(|store| {
        let todo = store
            .todos
            .get_mut(&name)
            .ok_or_else(|| anyhow::anyhow!("Todo '{name}' not found"))?;
        todo.toggle_done();
        Ok::<_, anyhow::Error>(todo.clone())
    })?;
    Ok(ok_json(todo))
}

async fn delete_todo(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.todo.write(|store| {
        store
            .todos
            .remove(&name)
            .ok_or_else(|| anyhow::anyhow!("Todo '{name}' not found"))
    })?;
    Ok(ok_json_message())
}
