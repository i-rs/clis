#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use owo_colors::OwoColorize;

pub mod assets;
pub mod routes;

pub use crate::config::DashboardConfig;

/// Shared application state for all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub core: Arc<RwLock<crate::core::AppCore>>,
    pub auth_token: String,
}

impl AppState {
    pub fn new(core: crate::core::AppCore, auth_token: String) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
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
    pub async fn run(self, core: crate::core::AppCore) {
        use axum::Router;

        let auth_token = if let Some(token) = self.config.auth_token.clone() {
            token
        } else {
            let token = uuid::Uuid::new_v4().to_string();
            if let Err(e) = Self::persist_auth_token(&token) {
                eprintln!("  {}  Failed to persist auth token: {}", "⚠".yellow(), e);
                println!(
                    "  {}  {} {} (not saved to config, will rotate on restart)",
                    "🔑".bright_blue(),
                    "Token:".bold().yellow(),
                    token.bright_white().bold()
                );
            } else {
                println!(
                    "  {}  {} {} (saved to config)",
                    "🔑".bright_blue(),
                    "Token:".bold().yellow(),
                    token.bright_white().bold()
                );
            }
            token
        };

        let state = AppState::new(core, auth_token.clone());

        let auth_middleware = axum::middleware::from_fn_with_state(state.clone(), auth_guard);

        let public_routes = Router::new().route("/api/health", axum::routing::get(routes::health));

        let api_routes = Router::new()
            .route(
                "/api/config",
                axum::routing::get(routes::get_config).patch(routes::update_config),
            )
            .route("/api/chat", axum::routing::post(routes::send_message))
            .route(
                "/api/chat/stream/{session_id}",
                axum::routing::get(routes::chat_stream),
            )
            .route(
                "/api/sessions/current",
                axum::routing::get(routes::get_current_session),
            )
            .route(
                "/api/sessions",
                axum::routing::get(routes::list_sessions).post(routes::create_session),
            )
            .route(
                "/api/sessions/{id}",
                axum::routing::get(routes::get_session).delete(routes::delete_session),
            )
            .route(
                "/api/sessions/{id}/switch",
                axum::routing::post(routes::switch_session),
            )
            .route(
                "/api/sessions/{id}/feedback",
                axum::routing::post(routes::post_session_feedback),
            )
            .route("/api/tools", axum::routing::get(routes::list_tools))
            .route("/api/images/{filename}", axum::routing::get(routes::serve_image))
            .route("/api/plugins", axum::routing::get(routes::list_plugins))
            .route("/api/skills", axum::routing::get(routes::list_skills))
            .route(
                "/api/guardrails/check",
                axum::routing::post(routes::check_guardrails),
            )
            .route(
                "/api/checkpoints",
                axum::routing::get(routes::list_checkpoints),
            )
            .route(
                "/api/memory/layered",
                axum::routing::get(routes::get_layered_memory)
                    .post(routes::clear_layered_memory),
            )
            .route(
                "/api/evals",
                axum::routing::get(routes::run_evals),
            )
            .route("/api/stats", axum::routing::get(routes::get_stats))
            .route(
                "/api/agents",
                axum::routing::get(routes::get_agents).post(routes::create_agent),
            )
            .route(
                "/api/agents/{id}",
                axum::routing::get(routes::get_agent_detail)
                    .put(routes::update_agent)
                    .delete(routes::delete_agent),
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
            format!("http://{}#{}", addr, auth_token)
                .underline()
                .bright_blue()
        );
        println!();

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind dashboard address");

        axum::serve(listener, app)
            .await
            .expect("Dashboard server error");
    }

    /// Persist the generated auth token to config.toml so it survives restarts.
    fn persist_auth_token(token: &str) -> anyhow::Result<()> {
        let mut cfg = crate::config::Config::load()?;
        cfg.dashboard.auth_token = Some(token.to_string());
        cfg.save()
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
            let mut resp =
                axum::response::IntoResponse::into_response(axum::Json(serde_json::json!({
                    "success": false,
                    "data": null,
                    "error": "Unauthorized",
                })));
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
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn test_state() -> AppState {
        let (_cfg, core) = crate::test_helpers::test_core();
        AppState {
            core: Arc::new(RwLock::new(core)),
            auth_token: "secret".to_string(),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_auth_valid_token() {
        let state = test_state();
        let app = Router::new()
            .route("/api/test", get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_guard,
            ))
            .with_state(state);

        let req = Request::builder()
            .uri("/api/test")
            .header("authorization", "Bearer secret")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_auth_missing_token() {
        let state = test_state();
        let app = Router::new()
            .route("/api/test", get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_guard,
            ))
            .with_state(state);

        let req = Request::builder()
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_auth_wrong_token() {
        let state = test_state();
        let app = Router::new()
            .route("/api/test", get(ok_handler))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_guard,
            ))
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
