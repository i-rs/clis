//! Serve mode — HTTP API server + Web Dashboard.
//!
//! `claw serve` is the default command. It starts an HTTP server that serves:
//! - REST API endpoints for chat, sessions, agents, stats, etc.
//! - SSE streaming for real-time chat responses
//! - Web Dashboard SPA (unless `--api-only` is set)

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use owo_colors::OwoColorize;

/// Shared application state for all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub core: Arc<RwLock<i_rs_claw_core::core::AppCore>>,
    pub auth_token: String,
}

impl AppState {
    pub fn new(core: i_rs_claw_core::core::AppCore, auth_token: String) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
            auth_token,
        }
    }
}

/// Run the serve-mode HTTP server. Blocks until shutdown.
pub async fn run(core: i_rs_claw_core::core::AppCore, host: String, port: u16, api_only: bool) {
    let config = core.config.clone();

    let auth_token = if let Some(token) = config.dashboard.auth_token.clone() {
        token
    } else {
        let token = uuid::Uuid::new_v4().to_string();
        if let Err(e) = persist_auth_token(&token) {
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

    let public_routes =
        axum::Router::new().route("/api/health", axum::routing::get(crate::dashboard::routes::health));

    let api_routes = axum::Router::new()
        .route(
            "/api/config",
            axum::routing::get(crate::dashboard::routes::get_config)
                .patch(crate::dashboard::routes::update_config),
        )
        .route(
            "/api/chat",
            axum::routing::post(crate::dashboard::routes::send_message),
        )
        .route(
            "/api/chat/stream/{session_id}",
            axum::routing::get(crate::dashboard::routes::chat_stream),
        )
        .route(
            "/api/sessions/current",
            axum::routing::get(crate::dashboard::routes::get_current_session),
        )
        .route(
            "/api/sessions",
            axum::routing::get(crate::dashboard::routes::list_sessions)
                .post(crate::dashboard::routes::create_session),
        )
        .route(
            "/api/sessions/{id}",
            axum::routing::get(crate::dashboard::routes::get_session)
                .delete(crate::dashboard::routes::delete_session),
        )
        .route(
            "/api/sessions/{id}/switch",
            axum::routing::post(crate::dashboard::routes::switch_session),
        )
        .route(
            "/api/sessions/{id}/feedback",
            axum::routing::post(crate::dashboard::routes::post_session_feedback),
        )
        .route(
            "/api/tools",
            axum::routing::get(crate::dashboard::routes::list_tools),
        )
        .route(
            "/api/images/{filename}",
            axum::routing::get(crate::dashboard::routes::serve_image),
        )
        .route(
            "/api/plugins",
            axum::routing::get(crate::dashboard::routes::list_plugins),
        )
        .route(
            "/api/skills",
            axum::routing::get(crate::dashboard::routes::list_skills),
        )
        .route(
            "/api/guardrails/check",
            axum::routing::post(crate::dashboard::routes::check_guardrails),
        )
        .route(
            "/api/checkpoints",
            axum::routing::get(crate::dashboard::routes::list_checkpoints),
        )
        .route(
            "/api/checkpoints/{id}",
            axum::routing::get(crate::dashboard::routes::get_checkpoint_detail),
        )
        .route(
            "/api/checkpoints/restore",
            axum::routing::post(crate::dashboard::routes::restore_checkpoint),
        )
        .route(
            "/api/memory/layered",
            axum::routing::get(crate::dashboard::routes::get_layered_memory)
                .post(crate::dashboard::routes::clear_layered_memory),
        )
        .route(
            "/api/memory/search",
            axum::routing::get(crate::dashboard::routes::search_layered_memory),
        )
        .route(
            "/api/evals",
            axum::routing::get(crate::dashboard::routes::run_evals),
        )
        .route(
            "/api/stats",
            axum::routing::get(crate::dashboard::routes::get_stats),
        )
        .route(
            "/api/agents",
            axum::routing::get(crate::dashboard::routes::get_agents)
                .post(crate::dashboard::routes::create_agent),
        )
        .route(
            "/api/agents/{id}",
            axum::routing::get(crate::dashboard::routes::get_agent_detail)
                .put(crate::dashboard::routes::update_agent)
                .delete(crate::dashboard::routes::delete_agent),
        )
        .route(
            "/api/providers",
            axum::routing::get(crate::dashboard::routes::list_providers),
        )
        .layer(auth_middleware);

    let app = if api_only {
        public_routes.merge(api_routes).with_state(state)
    } else {
        let static_routes = axum::Router::new()
            .route("/", axum::routing::get(crate::dashboard::assets::serve_root))
            .route(
                "/{*path}",
                axum::routing::get(crate::dashboard::assets::serve_assets),
            );
        public_routes
            .merge(api_routes)
            .merge(static_routes)
            .with_state(state)
    };

    let addr: SocketAddr = format!("{}:{}", host, port)
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
    if !api_only {
        println!(
            "  {}  {}   {}",
            "🔗".bright_blue(),
            "Dashboard".bold().bright_cyan(),
            format!("http://{}#{}", addr, auth_token)
                .underline()
                .bright_blue()
        );
    }
    println!();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind serve address");

    axum::serve(listener, app)
        .await
        .expect("Serve error");
}

fn persist_auth_token(token: &str) -> anyhow::Result<()> {
    use i_rs_claw_core::config::Config;
    let mut cfg = Config::load()?;
    cfg.dashboard.auth_token = Some(token.to_string());
    cfg.save()
}

// ── Auth middleware ──

/// Axum middleware that validates Bearer token and resolves user_id.
async fn auth_guard(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let user_id: String = match provided {
        Some(token) => {
            let core = state.core.read().await;
            if let Some(user) = core.config.dashboard.users.iter().find(|u| u.token == token) {
                user.id.clone()
            } else if state.auth_token == token {
                "default".to_string()
            } else {
                return unauthorized();
            }
        }
        None => return unauthorized(),
    };

    req.extensions_mut().insert(UserId(user_id));
    next.run(req).await
}

fn unauthorized() -> axum::response::Response {
    let mut resp = axum::response::IntoResponse::into_response(axum::Json(
        serde_json::json!({"success": false, "data": null, "error": "Unauthorized"}),
    ));
    *resp.status_mut() = axum::http::StatusCode::UNAUTHORIZED;
    resp.headers_mut().insert(
        axum::http::header::WWW_AUTHENTICATE,
        axum::http::HeaderValue::from_static("Bearer realm=\"claw-dashboard\""),
    );
    resp
}

/// Extractor: resolves the current user_id from request context.
#[derive(Clone, Debug)]
pub struct UserId(#[allow(dead_code)] pub String);

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for UserId {
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserId>()
            .cloned()
            .ok_or((axum::http::StatusCode::UNAUTHORIZED, "Missing user context"))
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

    fn run_auth_test<F>(name: &str, f: F)
    where
        F: FnOnce(tokio::runtime::Runtime, AppState) + Send + 'static,
    {
        let result = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("create test runtime");
                let state = test_state();
                f(rt, state);
            })
            .expect("spawn test thread")
            .join();
        if let Err(e) = result {
            if let Some(msg) = e.downcast_ref::<&str>() {
                panic!("test '{}' panicked: {}", name, msg);
            } else if let Some(msg) = e.downcast_ref::<String>() {
                panic!("test '{}' panicked: {}", name, msg);
            } else {
                panic!("test '{}' panicked (unknown)", name);
            }
        }
    }

    #[test]
    fn test_auth_valid_token() {
        run_auth_test("test_auth_valid_token", |rt, state| {
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
            let resp = rt.block_on(app.oneshot(req)).unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        });
    }

    #[test]
    fn test_auth_missing_token() {
        run_auth_test("test_auth_missing_token", |rt, state| {
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
            let resp = rt.block_on(app.oneshot(req)).unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }

    #[test]
    fn test_auth_wrong_token() {
        run_auth_test("test_auth_wrong_token", |rt, state| {
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
            let resp = rt.block_on(app.oneshot(req)).unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }
}
