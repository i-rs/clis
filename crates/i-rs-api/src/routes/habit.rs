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
async fn update_habit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.habit.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Habit '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_habits))
        .route("/", post(add_habit))
        .route("/{name}", get(get_habit).patch(update_habit))
        .route("/{name}/checkin", post(checkin_habit))
        .route("/{name}/stats", get(habit_stats))
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
    let records = state.habit.read(|store| {
        i_rs_habit::service::list_habits(store, None).map_err(ApiError::from)
    })?;
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
    let habit = state.habit.write(|store| {
        i_rs_habit::service::add_habit(store, name, description, frequency, tags, Vec::new()).map_err(ApiError::from)
    })?;
    Ok(ok_json(habit))
}

async fn get_habit(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let habit = state.habit.read(|store| {
        i_rs_habit::service::get_habit(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(habit))
}

async fn checkin_habit(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let habit = state.habit.write(|store| {
        i_rs_habit::service::checkin_habit(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(habit))
}

async fn habit_stats(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let habit = state.habit.read(|store| {
        i_rs_habit::service::get_habit(store, &name).map_err(ApiError::from)
    })?;
    Ok(ok_json(serde_json::json!({
        "name": habit.name,
        "checkin_count": habit.checkins.len(),
    })))
}
