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
        .route("/", get(list_bookmarks))
        .route("/", post(add_bookmark))
        .route("/:name", get(get_bookmark))
        .route("/:name", delete(delete_bookmark))
}

#[derive(Debug, Deserialize)]
pub struct AddBookmarkRequest {
    pub name: String,
    pub url: String,
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub tag: Option<String>,
}

async fn list_bookmarks(
    Query(params): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let tag = params.tag;
    call_service(move || i_rs_bookmark::service::list_bookmarks(tag)).await
}

async fn add_bookmark(
    Json(req): Json<AddBookmarkRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let name = req.name;
    let url = req.url;
    let tags = req.tag.unwrap_or_default();
    call_service(move || i_rs_bookmark::service::add_bookmark(name, url, None, None, tags, vec![])).await
}

async fn get_bookmark(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_bookmark::service::get_bookmark(&name)).await
}

async fn delete_bookmark(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service_unit(move || i_rs_bookmark::service::delete_bookmark(&name)).await
}
