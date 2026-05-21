#![allow(dead_code)]

use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::Mutex;

use owo_colors::OwoColorize;

pub mod assets;
pub mod routes;

pub use crate::config::DashboardConfig;

/// Shared application state for all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub core: Arc<Mutex<crate::core::AppCore>>,
    pub auth_token: String,
}

impl AppState {
    pub fn new(core: crate::core::AppCore, auth_token: String) -> Self {
        Self {
            core: Arc::new(Mutex::new(core)),
            auth_token,
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

        let auth_token = self.config.auth_token.clone().unwrap_or_else(|| {
            let token = uuid::Uuid::new_v4().to_string();
            println!(
                "  {}  {} {}",
                "🔑".bright_blue(),
                "Token:".bold().yellow(),
                token.bright_white().bold()
            );
            token
        });

        let state = AppState::new(inner, auth_token.clone());

        let auth_middleware = axum::middleware::from_fn_with_state(state.clone(), auth_guard);

        let public_routes = Router::new()
            .route("/api/health", axum::routing::get(routes::health));

        let api_routes = Router::new()
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
            )
            .layer(auth_middleware);

        let static_routes = Router::new()
            .route("/", axum::routing::get(assets::serve_root))
            .route("/{*path}", axum::routing::get(assets::serve_assets));

        let app = public_routes
            .merge(api_routes)
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
            format!("http://{}#{}", addr, auth_token).underline().bright_blue()
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

/// Axum middleware that validates Bearer token on API routes.
async fn auth_guard(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided {
        Some(token) if token == state.auth_token => next.run(req).await,
        _ => {
            tracing::warn!("Dashboard 认证失败: {}", req.uri().path());
            let mut resp = axum::response::IntoResponse::into_response(
                "{\"success\":false,\"data\":null,\"error\":\"Unauthorized\"}",
            );
            *resp.status_mut() = axum::http::StatusCode::UNAUTHORIZED;
            resp.headers_mut().insert(
                axum::http::header::WWW_AUTHENTICATE,
                axum::http::HeaderValue::from_static("Bearer realm=\"claw-dashboard\""),
            );
            resp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_auth_valid_token() {
        let state = AppState {
            core: Arc::new(Mutex::new(crate::test_helpers::test_core().1)),
            auth_token: "secret".to_string(),
        };
        let app = Router::new()
            .route("/api/test", get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(state.clone(), auth_guard))
            .with_state(state);

        let req = Request::builder()
            .uri("/api/test")
            .header("authorization", "Bearer secret")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_missing_token() {
        let state = AppState {
            core: Arc::new(Mutex::new(crate::test_helpers::test_core().1)),
            auth_token: "secret".to_string(),
        };
        let app = Router::new()
            .route("/api/test", get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(state.clone(), auth_guard))
            .with_state(state);

        let req = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_wrong_token() {
        let state = AppState {
            core: Arc::new(Mutex::new(crate::test_helpers::test_core().1)),
            auth_token: "secret".to_string(),
        };
        let app = Router::new()
            .route("/api/test", get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(state.clone(), auth_guard))
            .with_state(state);

        let req = Request::builder()
            .uri("/api/test")
            .header("authorization", "Bearer wrong")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
