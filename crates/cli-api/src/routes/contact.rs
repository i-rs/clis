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
async fn update_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.contact.write(|store| -> Result<_, ApiError> {
        let entry = store
            .entries
            .get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Contact '{id}' not found")))?;
        crate::update::merge_entry(entry, &body).map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_contacts))
        .route("/", post(add_contact))
        .route("/{id}", get(get_contact))
        .route("/{id}", delete(delete_contact).patch(update_contact))
}

#[derive(Debug, Deserialize)]
pub struct AddContactRequest {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub relationship: String,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
    pub last_contact: Option<String>,
    pub contact_count: u32,
}

async fn list_contacts(State(state): State<Arc<AppState>>) -> ApiResult<Json<serde_json::Value>> {
    let records = state.contact.read(|store| {
        let entries: Vec<_> = store.entries.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_contact(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddContactRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let phone = req.phone;
    let email = req.email;
    let relationship = req.relationship;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let last_contact = req
        .last_contact
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .transpose()?;
    let contact_count = req.contact_count;
    let now = chrono::Utc::now();
    let entry = i_rs_contact::models::Contact {
        name,
        phone,
        email,
        relationship,
        tags,
        remark,
        last_contact,
        contact_count,
        created_at: now,
        updated_at: now,
    };
    state.contact.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.contact.read(|store| {
        store
            .entries
            .get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Contact '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.contact.write(|store| {
        if store.entries.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Contact '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
