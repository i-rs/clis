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
        .route("/", get(list_notes))
        .route("/", post(add_note))
        .route("/:name", get(get_note))
        .route("/:name", delete(delete_note))
}

#[derive(Debug, Deserialize)]
pub struct AddNoteRequest {
    pub name: String,
    pub content: String,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
}

async fn list_notes(
    Query(params): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let tag = params.tag;
    call_service(move || i_rs_note::service::list_notes(tag)).await
}

async fn add_note(
    Json(req): Json<AddNoteRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let name = req.name;
    let title = None;
    let content = vec![req.content];
    let tags = req.tag.unwrap_or_default();
    call_service(move || i_rs_note::service::add_note(name, title, tags, content)).await
}

async fn get_note(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_note::service::get_note(&name)).await
}

async fn delete_note(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service_unit(move || i_rs_note::service::delete_note(&name)).await
}
