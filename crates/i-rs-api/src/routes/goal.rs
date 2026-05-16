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
async fn update_goal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.goal.write(|store| -> Result<_, ApiError> {
        let entry = store
            .goals
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Goal '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_goals))
        .route("/", post(add_goal))
        .route("/{id}", get(get_goal))
        .route("/{id}", delete(delete_goal).patch(update_goal))
}

#[derive(Debug, Deserialize)]
pub struct AddGoalRequest {
    pub name: String,
    pub target_amount: f64,
    pub deadline: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_goals(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.goal.read(|store| {
        let entries: Vec<_> = store.goals.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_goal(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddGoalRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let target_amount = req.target_amount;
    let parsed_deadline = chrono::DateTime::parse_from_rfc3339(&req.deadline)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let deadline = parsed_deadline.with_timezone(&chrono::Utc);
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let id = uuid::Uuid::new_v4().to_string();
    let entry = i_rs_goal::models::SavingsGoal {
        id: id.clone(),
        name,
        target_amount,
        current_amount: 0.0,
        deadline,
        milestones: Vec::new(),
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.goal.write(|store| {
        store.goals.insert(entry.id.clone(), entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_goal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.goal.read(|store| {
        store
            .goals
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Goal '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_goal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.goal.write(|store| {
        if store.goals.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Goal '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
