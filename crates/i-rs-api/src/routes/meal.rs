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
async fn update_meal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.meal.write(|store| -> Result<_, ApiError> {
        let entry = store.entries.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Meal '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_meals))
        .route("/", post(add_meal))
        .route("/{id}", get(get_meal))
        .route("/{id}", delete(delete_meal).patch(update_meal))
}

#[derive(Debug, Deserialize)]
pub struct AddMealRequest {
    pub meal_type: String,
    pub food_items: String,
    pub calories: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub date: String,
}

async fn list_meals(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.meal.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_meal(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddMealRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let meal_type = req.meal_type;
    let food_items = req.food_items;
    let calories = req.calories;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let date = chrono::NaiveDate::parse_from_str(&req.date, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;
    let entry = i_rs_meal::models::MealEntry::new(meal_type, food_items, calories, tags, remark, date);
    state.meal.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_meal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.meal.read(|store| {
        store.entries.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Meal '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_meal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.meal.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Meal '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
