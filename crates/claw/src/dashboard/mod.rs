#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use owo_colors::OwoColorize;

pub mod assets;
pub mod routes;

pub use crate::config::DashboardConfig;

/// Shared application state for all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub core: Arc<Mutex<crate::core::AppCore>>,
}

impl AppState {
    pub fn new(core: crate::core::AppCore) -> Self {
        Self {
            core: Arc::new(Mutex::new(core)),
        }
    }
}

/// Dashboard web server providing a REST API and SPA frontend.
pub struct Dashboard {
    config: DashboardConfig,
}

impl Dashboard {
    pub fn new(config: DashboardConfig) -> Self {
        Self { config }
    }

    /// Run the dashboard server. Blocks until shutdown.
    pub async fn run(self, core: Arc<crate::core::AppCore>) {
        use axum::Router;

        let inner = Arc::into_inner(core).expect("AppCore must have exactly one reference");
        let state = AppState::new(inner);
        // Routes: API + static files
        let api_routes = Router::new()
            .route("/api/health", axum::routing::get(routes::health))
            .route("/api/config", axum::routing::get(routes::get_config))
            .route("/api/chat", axum::routing::post(routes::send_message))
            .route(
                "/api/chat/stream/{session_id}",
                axum::routing::get(routes::chat_stream),
            )
            .route("/api/sessions/current", axum::routing::get(routes::get_current_session))
            .route("/api/sessions", axum::routing::get(routes::list_sessions).post(routes::create_session))
            .route(
                "/api/sessions/{id}",
                axum::routing::get(routes::get_session).delete(routes::delete_session),
            )
            .route(
                "/api/sessions/{id}/switch",
                axum::routing::post(routes::switch_session),
            )
            .route("/api/tools", axum::routing::get(routes::list_tools))
            .route("/api/plugins", axum::routing::get(routes::list_plugins))
            .route("/api/skills", axum::routing::get(routes::list_skills))
            .route("/api/agents", axum::routing::get(routes::get_agents).post(routes::create_agent))
            .route(
                "/api/agents/{id}",
                axum::routing::get(routes::get_agent_detail).put(routes::update_agent).delete(routes::delete_agent),
            );

        // Static frontend routes (SPA)
        let static_routes = Router::new()
            .route("/", axum::routing::get(assets::serve_root))
            .route("/{*path}", axum::routing::get(assets::serve_assets));

        let app = api_routes
            .merge(static_routes)
            .with_state(state);

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .expect("Invalid dashboard address");

        println!(
            "  {}  {}  http://{}/api/health",
            "📡".bright_blue(),
            "API".bold().bright_cyan(),
            addr
        );
        println!(
            "  {}  {} http://{}/api/chat",
            "💬".bright_blue(),
            "Chat".bold().bright_cyan(),
            addr
        );
        println!(
            "  {}  {}	{}",
            "🔗".bright_blue(),
            "Dashboard".bold().bright_cyan(),
            format!("http://{}", addr).underline().bright_blue()
        );
        println!();

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind dashboard address");

        axum::serve(listener, app)
            .await
            .expect("Dashboard server error");
    }
}
