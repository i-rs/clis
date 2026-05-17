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
async fn update_birthday(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.birthday.write(|store| -> Result<_, ApiError> {
        let entry = store
            .birthdays
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Birthday '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_birthdays))
        .route("/", post(add_birthday))
        .route("/{id}", get(get_birthday))
        .route("/{id}", delete(delete_birthday).patch(update_birthday))
}

#[derive(Debug, Deserialize)]
pub struct AddBirthdayRequest {
    pub name: String,
    pub birth_date: String,
    pub year: Option<i32>,
    pub relationship: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_birthdays(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.birthday.read(|store| {
        let entries: Vec<_> = store.birthdays.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_birthday(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddBirthdayRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let birth_date = req.birth_date;
    let year = req.year;
    let relationship = req.relationship;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_birthday::models::Birthday {
        name,
        birth_date,
        year,
        relationship,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.birthday.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_birthday(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.birthday.read(|store| {
        store
            .birthdays
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Birthday '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_birthday(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.birthday.write(|store| {
        if store.birthdays.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Birthday '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
