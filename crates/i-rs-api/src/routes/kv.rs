use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, Query},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::{call_service, call_service_unit};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_kv))
        .route("/:key", get(get_kv))
        .route("/:key", post(set_kv))
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
    let tag = query.search; // API uses "search" but KV service uses tag filtering
    call_service(move || i_rs_kv::service::list_kv(tag)).await
}

async fn set_kv(
    Path(key): Path<String>,
    Json(req): Json<SetKvRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let key = key;
    let value = req.value;
    call_service(move || i_rs_kv::service::add_kv(key, value, vec![], vec![])).await
}

async fn get_kv(
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_kv::service::get_kv(&key)).await
}

async fn delete_kv(
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service_unit(move || i_rs_kv::service::delete_kv(&key)).await
}
