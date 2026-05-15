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
        .route("/", get(list_kv))
        .route("/", post(set_kv))
        .route("/:key", get(get_kv))
        .route("/:key", delete(delete_kv))
}

#[derive(Debug, Deserialize)]
pub struct SetKvRequest {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ListKvQuery {
    pub search: Option<String>,
}

async fn list_kv(
    Query(query): Query<ListKvQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["list".to_string()];
    if let Some(ref search) = query.search {
        args.push("--search".to_string());
        args.push(search.clone());
    }
    run_cli("i-rs-kv", args).await
}

async fn set_kv(
    Path(key): Path<String>,
    Json(req): Json<SetKvRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-kv", vec!["add".to_string(), key, req.value]).await
}

async fn get_kv(
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-kv", vec!["get".to_string(), key]).await
}

async fn delete_kv(
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-kv", vec!["delete".to_string(), key]).await
}
