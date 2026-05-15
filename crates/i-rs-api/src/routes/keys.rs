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
    let tag = query.search;
    call_service(move || i_rs_keys::service::list_keys(tag)).await
}

async fn add_key(
    Path(name): Path<String>,
    Json(req): Json<AddKeyRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let name = name;
    let value = req.value;
    let remark = req.remark.unwrap_or_default();
    call_service(move || i_rs_keys::service::add_key(name, "api_key".to_string(), value, vec![], vec![remark])).await
}

async fn get_key(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_keys::service::get_key(&name)).await
}

async fn delete_key(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service_unit(move || i_rs_keys::service::delete_key(&name)).await
}
