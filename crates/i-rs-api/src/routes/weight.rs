use axum::{
    Router,
    routing::{get, post},
    extract::Path,
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::run_cli;

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_weights))
        .route("/", post(add_weight))
        .route("/stats", get(weight_stats))
        .route("/:date", get(get_weight))
}

#[derive(Debug, Deserialize)]
pub struct AddWeightRequest {
    pub date: Option<String>,
    pub weight: f64,
    pub remark: Option<Vec<String>>,
}

async fn list_weights() -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-weight", vec!["list".to_string()]).await
}

async fn add_weight(
    Json(req): Json<AddWeightRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let mut args = vec!["add".to_string()];
    if let Some(ref date) = req.date {
        args.push("--date".to_string());
        args.push(date.clone());
    }
    args.push(req.weight.to_string());
    if let Some(ref remarks) = req.remark {
        for remark in remarks {
            args.push("--remark".to_string());
            args.push(remark.clone());
        }
    }

    run_cli("i-rs-weight", args).await
}

async fn get_weight(
    Path(date): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-weight", vec!["get".to_string(), date]).await
}

async fn weight_stats() -> Result<Json<serde_json::Value>, impl IntoResponse> {
    run_cli("i-rs-weight", vec!["stats".to_string()]).await
}
