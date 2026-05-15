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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_movies))
        .route("/", post(add_movie))
        .route("/{id}", get(get_movie))
        .route("/{id}", delete(delete_movie))
}

#[derive(Debug, Deserialize)]
pub struct AddMovieRequest {
    pub name: String,
    pub year: Option<i32>,
    pub director: Option<String>,
    pub watched: bool,
    pub rating: Option<serde_json::Value>,
    pub review: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub remark: Option<Vec<String>>,
}

async fn list_movies(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let records = state.movie.read(|store| {
        let entries: Vec<_> = store.movies.values().cloned().collect();
        Ok::<_, ApiError>(entries)
    })?;
    Ok(ok_json_list(records))
}

async fn add_movie(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddMovieRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let name = req.name;
    let year = req.year;
    let director = req.director;
    let watched = req.watched;
    let rating: Option<f32> = req.rating
        .map(|v| serde_json::from_value(v)
            .map_err(|e| ApiError::BadRequest(format!("Invalid rating: {e}"))))
        .transpose()?;
    let review = req.review.unwrap_or_default();
    let release_date = req.release_date
        .map(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|_| ApiError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))
        ).transpose()?;
    let tags = req.tags.unwrap_or_default();
    let remark = req.remark.unwrap_or_default();
    let now = chrono::Utc::now();
    let entry = i_rs_movie::models::Movie {
        name,
        year,
        director,
        watched,
        rating,
        review,
        release_date,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };
    state.movie.write(|store| {
        store.add_entry(entry.clone());
    });
    Ok(ok_json(entry))
}

async fn get_movie(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let entry = state.movie.read(|store| {
        store.movies.get(&id).cloned().ok_or_else(|| ApiError::NotFound(format!("Movie '{id}' not found")))
    })?;
    Ok(ok_json(entry))
}

async fn delete_movie(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    state.movie.write(|store| {
        if store.movies.remove(&id).is_none() {
            return Err(ApiError::NotFound(format!("Movie '{id}' not found")));
        }
        Ok(())
    })?;
    Ok(ok_json_message())
}
