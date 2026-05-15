use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use crate::api::{ok_json, ok_json_list, ok_json_message};
use crate::response::{ApiError, ApiResult};
use crate::AppState;
async fn update_exercise(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.exercise.write(|store| -> Result<_, ApiError> {
        let entry = store.records.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Exercise '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_exercises))
        .route("/", post(add_exercise))
        .route("/{id}", get(get_exercise))
        .route("/{id}", delete(delete_exercise).patch(update_exercise))
}

#[derive(Debug, Deserialize)]
pub struct AddExerciseRecordRequest {
    pub name: String,
    pub exercise_type: String,
    pub duration_minutes: u32,
    pub calories: Option<u32>,
    pub notes: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_exercises(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.exercise.read(|store| {
        let entries: Vec<_> = store.records.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_exercise(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddExerciseRecordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let exercise_type = req.exercise_type;
    let duration_minutes = req.duration_minutes;
    let calories = req.calories;
    let notes = req.notes.unwrap_or_default();
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_exercise::models::ExerciseRecord {
        name,
        exercise_type,
        duration_minutes,
        calories,
        notes,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.exercise.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_exercise(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.exercise.read(|store| {
        store.records.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Exercise '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_exercise(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.exercise.write(|store| {
        if store.records.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Exercise '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
