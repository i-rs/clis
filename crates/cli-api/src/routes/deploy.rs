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
async fn update_deploy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.deploy.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Deploy '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_deploys))
        .route("/", post(add_deploy))
        .route("/{id}", get(get_deploy))
        .route("/{id}", delete(delete_deploy).patch(update_deploy))
}

#[derive(Debug, Deserialize)]
pub struct AddDeployRecordRequest {
    pub project: String,
    pub environment: String,
    pub version: String,
    pub status: String,
    pub deployed_at: String,
    pub rollback_from: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_deploys(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.deploy.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_deploy(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddDeployRecordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let project = req.project;
    let environment = req.environment;
    let version = req.version;
    let status: i_rs_deploy::models::DeployStatus =
        serde_json::from_value(serde_json::json!(req.status))
            .map_err(|e| ApiError::BadRequest(format!("Invalid status: {e}")))?;
    let parsed_deployed_at = chrono::DateTime::parse_from_rfc3339(&req.deployed_at)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let deployed_at = parsed_deployed_at.with_timezone(&chrono::Utc);
    let rollback_from = req.rollback_from;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_deploy::models::DeployRecord {
        id: id.clone(),
        project,
        environment,
        version,
        status,
        deployed_at,
        rollback_from,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.deploy.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_deploy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.deploy.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Deploy '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_deploy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.deploy.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Deploy '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
