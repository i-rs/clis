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
        .route("/", get(list_domains))
        .route("/", post(add_domain))
        .route("/{id}", get(get_domain))
        .route("/{id}", delete(delete_domain))
}

#[derive(Debug, Deserialize)]
pub struct AddDomainRequest {
    pub name: String,
    pub expiry_date: String,
    pub registrar: Option<String>,
    pub password: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_domains(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.domain.read(|store| {
        let entries: Vec<_> = store.domains.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_domain(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddDomainRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let parsed_expiry_date = chrono::DateTime::parse_from_rfc3339(&req.expiry_date)
        .map_err(|_| ApiError::BadRequest("Invalid datetime, expected RFC3339".to_string()))?;
    let expiry_date = parsed_expiry_date.with_timezone(&chrono::Utc);
    let registrar = req.registrar;
    let password = req.password;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_domain::models::Domain {
        name,
        expiry_date,
        registrar,
        password,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.domain.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_domain(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.domain.read(|store| {
        store.domains.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Domain '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_domain(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.domain.write(|store| {
        if store.domains.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Domain '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
