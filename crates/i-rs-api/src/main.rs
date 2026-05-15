mod api;
mod response;
mod routes;
mod store;

use std::sync::Arc;

use axum::{
    Router,
    http::{Method, header},
};
use tower_http::cors::{CorsLayer, Any};

use store::SharedStore;

pub struct AppState {
    pub todo: SharedStore<i_rs_todo::models::TodoStore>,
    pub weight: SharedStore<i_rs_weight::models::WeightStore>,
    pub mood: SharedStore<i_rs_mood::models::MoodStore>,
    pub habit: SharedStore<i_rs_habit::models::HabitStore>,
    pub note: SharedStore<i_rs_note::models::NoteStore>,
    pub bookmark: SharedStore<i_rs_bookmark::models::BookmarkStore>,
    pub kv: SharedStore<i_rs_kv::models::KvStore>,
    pub keys: SharedStore<i_rs_keys::models::KeyStore>,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

    let state = Arc::new(AppState {
        todo: SharedStore::load("todo"),
        weight: SharedStore::load("weights"),
        mood: SharedStore::load("mood"),
        habit: SharedStore::load("habit"),
        note: SharedStore::load("note"),
        bookmark: SharedStore::load("bookmark"),
        kv: SharedStore::load("kv"),
        keys: SharedStore::load("keys"),
    });

    let app = Router::new()
        .route("/", axum::routing::get(handlers::health))
        .nest("/api/todo", routes::todo::router())
        .nest("/api/weight", routes::weight::router())
        .nest("/api/habit", routes::habit::router())
        .nest("/api/note", routes::note::router())
        .nest("/api/bookmark", routes::bookmark::router())
        .nest("/api/mood", routes::mood::router())
        .nest("/api/kv", routes::kv::router())
        .nest("/api/keys", routes::keys::router())
        .with_state(state)
        .layer(cors);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("🚀 i-rs-api server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address. Is port 8080 already in use?");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

mod handlers {
    use axum::Json;

    pub async fn health() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "success": true,
            "status": "ok",
            "service": "i-rs-api",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "REST API for i-rs CLI tools",
            "endpoints": [
                "/api/todo",
                "/api/weight",
                "/api/habit",
                "/api/note",
                "/api/bookmark",
                "/api/mood",
                "/api/kv",
                "/api/keys"
            ]
        }))
    }
}
