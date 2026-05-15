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
async fn update_server(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.server.write(|store| -> Result<_, ApiError> {
        let entry = store.servers.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("Server '{id}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        Ok(entry.clone())
    })?;
    Ok(ok_json(entry))
}



pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_servers))
        .route("/", post(add_server))
        .route("/{id}", get(get_server))
        .route("/{id}", delete(delete_server).patch(update_server))
}

#[derive(Debug, Deserialize)]
pub struct AddServerRequest {
    pub name: String,
    pub host: String,
    pub port: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_servers(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.server.read(|store| {
        let entries: Vec<_> = store.servers.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_server(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddServerRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let host = req.host;
    let port: u16 = req.port.parse()
        .map_err(|e| ApiError::BadRequest(format!("Invalid port: {e}")))?;
    let user = req.user;
    let password = req.password;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_server::models::Server {
        name,
        host,
        port,
        user,
        password,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.server.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_server(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.server.read(|store| {
        store.servers.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Server '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_server(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.server.write(|store| {
        if store.servers.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Server '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
