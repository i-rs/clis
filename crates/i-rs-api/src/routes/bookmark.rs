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
    let mut args = vec!["list".to_string()];
    if let Some(ref tag) = params.tag {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }

    run_cli("i-rs-bookmark", args).await
}

async fn add_bookmark(
    Json(req): Json<AddBookmarkRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["add".to_string(), req.name.clone()];
    args.push("--url".to_string());
    args.push(req.url.clone());
    for tag in req.tag.iter().flatten() {
        args.push("--tag".to_string());
        args.push(tag.clone());
    }

    run_cli("i-rs-bookmark", args).await
}

async fn get_bookmark(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-bookmark", vec!["get".to_string(), name]).await
}

async fn delete_bookmark(
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-bookmark", vec!["delete".to_string(), name]).await
}
