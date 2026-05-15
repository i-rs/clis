use axum::{
    Router,
    routing::{get, post},
    extract::Path,
    Json,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::api::call_service;

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
    call_service(|| i_rs_weight::service::list_weights(None)).await
}

async fn add_weight(
    Json(req): Json<AddWeightRequest>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let date = req.date.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    let weight = req.weight;
    let remark = req.remark.unwrap_or_default();
    call_service(move || i_rs_weight::service::add_weight(date, weight, remark)).await
}

async fn get_weight(
    Path(date): Path<String>,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(move || i_rs_weight::service::get_weight(&date)).await
}

async fn weight_stats() -> Result<Json<serde_json::Value>, impl IntoResponse> {
    call_service(|| -> anyhow::Result<serde_json::Value> {
        let mut storage = i_rs_core::Storage::<i_rs_weight::models::WeightStore>::new("weights");
        storage.load()?;
        Ok(serde_json::json!({ "count": storage.data.records.len() }))
    }).await
}
