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
async fn update_plant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.plant.write(|store| -> Result<_, ApiError> {
        let entry = store.get_entry_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Plant '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_plants))
        .route("/", post(add_plant))
        .route("/{id}", get(get_plant))
        .route("/{id}", delete(delete_plant).patch(update_plant))
}

#[derive(Debug, Deserialize)]
pub struct AddPlantRequest {
    pub name: String,
    pub species: String,
    pub location: String,
    pub watering_interval_days: u32,
}

async fn list_plants(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.plant.read(|store| {
        let entries: Vec<_> = store.plants.to_vec();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_plant(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddPlantRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let species = req.species;
    let location = req.location;
    let watering_interval_days = req.watering_interval_days;
    let entry = i_rs_plant::models::Plant::new(name, species, location, watering_interval_days);
    state.plant.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_plant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.plant.read(|store| {
        store.plants.iter().find(|e| e.name == id).cloned().ok_or_else(|| ApiError::NotFound(format!("Plant '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_plant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.plant.write(|store| {
        let len = store.plants.len();
        store.plants.retain(|e| e.name != id);
        if store.plants.len() == len {
            return Err(ApiError::NotFound(format!("Plant '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
