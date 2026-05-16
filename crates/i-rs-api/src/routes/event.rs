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
async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.event.write(|store| -> Result<_, ApiError> {
        let entry = store
            .events
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Event '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_events))
        .route("/", post(add_event))
        .route("/{id}", get(get_event))
        .route("/{id}", delete(delete_event).patch(update_event))
}

#[derive(Debug, Deserialize)]
pub struct AddEventRequest {
    pub name: String,
    pub date: String,
}

async fn list_events(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.event.read(|store| {
        let entries: Vec<_> = store.events.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_event(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddEventRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let date = chrono::DateTime::parse_from_rfc3339(&req.date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime format, expected RFC3339".to_string()))?
        .with_timezone(&chrono::Utc);
    let entry = i_rs_event::models::Event::new(name, date);
    state.event.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.event.read(|store| {
        store
            .events
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Event '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.event.write(|store| {
        if store.events.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Event '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
