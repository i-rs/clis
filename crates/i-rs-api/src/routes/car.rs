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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_cars))
        .route("/", post(add_car))
        .route("/{id}", get(get_car))
        .route("/{id}", delete(delete_car))
}

#[derive(Debug, Deserialize)]
pub struct AddCarRequest {
    pub name: String,
    pub license_plate: String,
    pub brand: String,
    pub model: String,
    pub mileage: f64,
}

async fn list_cars(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.car.read(|store| {
        let entries: Vec<_> = store.cars.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_car(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddCarRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let license_plate = req.license_plate;
    let brand = req.brand;
    let model = req.model;
    let mileage = req.mileage;
    let entry = i_rs_car::models::Car::new(name, license_plate, brand, model, mileage);
    state.car.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_car(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.car.read(|store| {
        store.cars.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Car '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_car(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.car.write(|store| {
        if store.cars.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Car '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
