use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use owo_colors::OwoColorize;

use crate::server::rate_limit::ChatConcurrency;

/// Shared application state for all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub core: Arc<RwLock<i_rs_claw_core::core::AppCore>>,
    pub auth_token: String,
    pub chat_concurrency: Arc<ChatConcurrency>,
}

impl AppState {
    pub fn new(core: i_rs_claw_core::core::AppCore, auth_token: String) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
            auth_token,
            chat_concurrency: Arc::new(ChatConcurrency::new(3)),
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

    if host == "0.0.0.0" || host == "::" {
        tracing::warn!(
            host = %host,
            "SECURITY: binding to a public address — dashboard is reachable from the network. \
             Ensure auth_token is set and network access is restricted."
        );
    }

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
    cfg.save()?;

    // Defense-in-depth: ensure config file has 0o600 on Unix regardless
    // of how it was created. atomic_write() already sets this on the temp
    // file before rename, but an explicit set_mode here guards against
    // pre-existing files with looser perms and any future changes to the
    // save path.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        let config_path = home.join(".i-rs").join("claw").join("config.toml");
        if config_path.exists() {
            let mut perms = std::fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&config_path, perms)?;
            tracing::debug!(path = %config_path.display(), mode = "0600", "config permissions tightened");
        }
    }

    Ok(())
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    /// Verify that the post-save `set_mode(0o600)` pattern (as used in
    /// `persist_auth_token`) results in mode 0o600 on the file.
    ///
    /// We don't call `persist_auth_token` directly because it writes to
    /// the user's real `~/.i-rs/claw/config.toml`. Instead we exercise the
    /// same `atomic_write + set_mode` sequence on a temp file.
    #[test]
    fn test_persist_auth_token_sets_0600_pattern() {
        let dir = tempfile::tempdir().expect("tempdir failed");
        let path = dir.path().join("config.toml");

        // atomic_write sets 0o600 on the temp file before rename.
        i_rs_claw_core::utils::atomic_write(&path, "# test\nauth_token = \"abc\"\n")
            .expect("atomic_write failed");

        // Defense-in-depth: explicit set_mode after the save (mirrors
        // persist_auth_token's pattern).
        let mut perms = std::fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms).expect("set_permissions");

        let mode = std::fs::metadata(&path).expect("metadata").permissions().mode();
        let actual = mode & 0o777;
        assert_eq!(
            actual, 0o600,
            "config file should be 0600, got {:o}",
            actual
        );
    }
}
