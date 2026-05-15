mod api;
mod handlers;
mod routes;

use axum::{
    Router,
    http::{Method, header},
};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

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
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("🚀 i-rs-api server starting on http://{}", addr);
    println!("📖 API docs: http://{}/api/docs", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
