use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::AppState;
use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};
async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.project.write(|store| -> Result<_, ApiError> {
        let entry = store
            .get_entry_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Project '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_projects))
        .route("/", post(add_project))
        .route("/{id}", get(get_project))
        .route("/{id}", delete(delete_project).patch(update_project))
}

#[derive(Debug, Deserialize)]
pub struct AddProjectRequest {
    pub name: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub milestones: Option<serde_json::Value>,
    pub tasks: Option<serde_json::Value>,
}

async fn list_projects(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.project.read(|store| {
        let entries: Vec<_> = store.projects.to_vec();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_project(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddProjectRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let description = req.description;
    let status: i_rs_project::models::ProjectStatus =
        serde_json::from_value(serde_json::json!(req.status))
            .map_err(|e| ApiError::BadRequest(format!("Invalid status: {e}")))?;
    let priority: i_rs_project::models::Priority =
        serde_json::from_value(serde_json::json!(req.priority))
            .map_err(|e| ApiError::BadRequest(format!("Invalid priority: {e}")))?;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let milestones: Vec<i_rs_project::models::Milestone> = req
        .milestones
        .map(|v| {
            serde_json::from_value(v)
                .map_err(|e| ApiError::BadRequest(format!("Invalid milestones: {e}")))
        })
        .transpose()?
        .unwrap_or_default();
    let tasks: Vec<i_rs_project::models::Task> = req
        .tasks
        .map(|v| {
            serde_json::from_value(v)
                .map_err(|e| ApiError::BadRequest(format!("Invalid tasks: {e}")))
        })
        .transpose()?
        .unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_project::models::Project {
        name,
        description,
        status,
        priority,
        tags,
        remark,
        milestones,
        tasks,
        created_at: now,
        updated_at: now,
    };
    state.project.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.project.read(|store| {
        store
            .projects
            .iter()
            .find(|e| e.name == id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Project '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.project.write(|store| {
        let len = store.projects.len();
        store.projects.retain(|e| e.name != id);
        if store.projects.len() == len {
            return Err(ApiError::NotFound(format!("Project '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
