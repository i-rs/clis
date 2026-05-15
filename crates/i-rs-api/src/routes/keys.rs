use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, Query},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::run_cli;

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_keys))
        .route("/", post(add_key))
        .route("/:name", get(get_key))
        .route("/:name", delete(delete_key))
}

#[derive(Debug, Deserialize)]
pub struct AddKeyRequest {
    pub value: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListKeysQuery {
    pub search: Option<String>,
}

async fn list_keys(
    Query(query): Query<ListKeysQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["list".to_string()];
    if let Some(ref search) = query.search {
        args.push("--search".to_string());
        args.push(search.clone());
    }
    run_cli("i-rs-keys", args).await
}

async fn add_key(
    Path(name): Path<String>,
    Json(req): Json<AddKeyRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["add".to_string(), name, req.value];
    if let Some(ref remark) = req.remark {
        args.push("--remark".to_string());
        args.push(remark.clone());
    }
    run_cli("i-rs-keys", args).await
}

async fn get_key(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-keys", vec!["get".to_string(), name]).await
}

async fn delete_key(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-keys", vec!["delete".to_string(), name]).await
}
