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
    let auth_middleware =
        axum::middleware::from_fn_with_state(state.clone(), crate::server::middleware::auth_guard);

    let public_routes =
        axum::Router::new().route("/api/health", axum::routing::get(crate::server::routes::health));

    let api_routes = axum::Router::new()
        .route("/api/config", axum::routing::get(crate::server::routes::get_config).patch(crate::server::routes::update_config))
        .route("/api/chat", axum::routing::post(crate::server::routes::chat))
        .route("/api/chat/stream/{session_id}", axum::routing::get(crate::server::routes::chat_stream))
        .route("/api/chat/stream/{session_id}/resume", axum::routing::get(crate::server::routes::chat_stream_resume))
        .route("/api/sessions/current", axum::routing::get(crate::server::routes::get_current_session))
        .route("/api/sessions", axum::routing::get(crate::server::routes::list_sessions).post(crate::server::routes::create_session))
        .route("/api/sessions/{id}", axum::routing::get(crate::server::routes::get_session).delete(crate::server::routes::delete_session))
        .route("/api/sessions/{id}/switch", axum::routing::post(crate::server::routes::switch_session))
        .route("/api/sessions/{id}/feedback", axum::routing::post(crate::server::routes::post_session_feedback))
        .route("/api/tools", axum::routing::get(crate::server::routes::list_tools))
        .route("/api/images/{filename}", axum::routing::get(crate::server::routes::serve_image))
        .route("/api/plugins", axum::routing::get(crate::server::routes::list_plugins))
        .route("/api/skills", axum::routing::get(crate::server::routes::list_skills))
        .route("/api/guardrails/check", axum::routing::post(crate::server::routes::check_guardrails))
        .route("/api/checkpoints", axum::routing::get(crate::server::routes::list_checkpoints))
        .route("/api/checkpoints/{id}", axum::routing::get(crate::server::routes::get_checkpoint_detail))
        .route("/api/checkpoints/restore", axum::routing::post(crate::server::routes::restore_checkpoint))
        .route("/api/memory/layered", axum::routing::get(crate::server::routes::get_layered_memory).post(crate::server::routes::clear_layered_memory))
        .route("/api/memory/search", axum::routing::get(crate::server::routes::search_layered_memory))
        .route("/api/evals", axum::routing::get(crate::server::routes::run_evals))
        .route("/api/stats", axum::routing::get(crate::server::routes::get_stats))
        .route("/api/agents", axum::routing::get(crate::server::routes::get_agents).post(crate::server::routes::create_agent))
        .route("/api/agents/{id}", axum::routing::get(crate::server::routes::get_agent_detail).put(crate::server::routes::update_agent).delete(crate::server::routes::delete_agent))
        .route("/api/providers", axum::routing::get(crate::server::routes::list_providers).post(crate::server::routes::create_provider))
        .route("/api/providers/{name}", axum::routing::put(crate::server::routes::update_provider).delete(crate::server::routes::delete_provider))
        .route("/api/users", axum::routing::get(crate::server::routes::list_users).post(crate::server::routes::create_user))
        .route("/api/users/{id}", axum::routing::delete(crate::server::routes::delete_user))
        .route("/api/mcp", axum::routing::get(crate::server::routes::list_mcp_configs).post(crate::server::routes::create_mcp_config))
        .route("/api/mcp/{name}", axum::routing::put(crate::server::routes::update_mcp_config).delete(crate::server::routes::delete_mcp_config))
        .route("/api/settings", axum::routing::get(crate::server::routes::list_settings).put(crate::server::routes::set_setting))
        .route("/api/settings/{key}", axum::routing::get(crate::server::routes::get_setting).put(crate::server::routes::set_setting_by_key).delete(crate::server::routes::delete_setting))
        .layer(auth_middleware);

    let app = if api_only {
        public_routes.merge(api_routes).with_state(state)
    } else {
        let static_routes = axum::Router::new()
            .route("/", axum::routing::get(crate::server::assets::serve_root))
            .route("/{*path}", axum::routing::get(crate::server::assets::serve_assets));
        public_routes.merge(api_routes).merge(static_routes).with_state(state)
    };

    let addr: SocketAddr = format!("{}:{}", host, port).parse().expect("Invalid address");
    println!("  {}  {}  http://{}/api/health", "📡".bright_blue(), "API".bold().bright_cyan(), addr);
    println!("  {}  {} http://{}/api/chat", "💬".bright_blue(), "Chat".bold().bright_cyan(), addr);
    if !api_only {
        println!("  {}  {}   {}", "🔗".bright_blue(), "Dashboard".bold().bright_cyan(), format!("http://{}#{}", addr, auth_token).underline().bright_blue());
    }
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind serve address");
    axum::serve(listener, app).await.expect("Serve error");
}

fn persist_auth_token(token: &str) -> anyhow::Result<()> {
    let mut cfg = i_rs_claw_core::config::Config::load()?;
    cfg.dashboard.auth_token = Some(token.to_string());
    cfg.save()
}
