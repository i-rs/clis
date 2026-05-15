use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use crate::api::{ok_json, ok_json_list};
use crate::response::{ApiError, ApiResult};
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
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
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub tag: Option<Vec<String>>,
}

async fn list_habits(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records: Vec<i_rs_habit::models::ListItem> = state.habit.read(|store| {
        store.entries.values().map(|h| h.into()).collect()
    });
    Ok(ok_json_list(records))
}

async fn add_habit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddHabitRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let description = req.description.unwrap_or_default();
    let frequency = req.frequency.unwrap_or_else(|| "daily".to_string());
    let tags = req.tag.unwrap_or_default();

    let exists = state.habit.read(|store| store.entries.contains_key(&name));
    if exists {
        return Err(ApiError::Conflict(format!("Habit '{name}' already exists")));
    }

    let now = chrono::Utc::now();
    let habit = state.habit.write(|store| {
        let habit = i_rs_habit::models::Habit {
            name: name.clone(),
            description,
            frequency,
            tags,
            remark: Vec::new(),
            checkins: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        store.add_entry(habit.clone());
        habit
    });
    Ok(ok_json(habit))
}

async fn get_habit(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let habit = state
        .habit
        .read(|store| store.entries.get(&name).cloned())
        .ok_or_else(|| ApiError::NotFound(format!("Habit '{name}' not found")))?;
    Ok(ok_json(habit))
}

async fn checkin_habit(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let habit = state.habit.write(|store| {
        let habit = store
            .entries
            .get_mut(&name)
            .ok_or_else(|| anyhow::anyhow!("Habit '{name}' not found"))?;
        habit.checkins.push(i_rs_habit::models::Checkin {
            date: chrono::Utc::now(),
        });
        habit.updated_at = chrono::Utc::now();
        Ok::<_, anyhow::Error>(habit.clone())
    })?;
    Ok(ok_json(habit))
}

async fn habit_stats(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let stats = state.habit.read(|store| {
        store
            .entries
            .get(&name)
            .map(|h| (h.name.clone(), h.checkins.len()))
            .ok_or_else(|| ApiError::NotFound(format!("Habit '{name}' not found")))
    })?;
    Ok(ok_json(serde_json::json!({
        "name": stats.0,
        "checkin_count": stats.1,
    })))
}
