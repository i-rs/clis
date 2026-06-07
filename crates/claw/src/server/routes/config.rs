use crate::server::AppState;
use axum::{
    Json,
    extract::State,
};
use serde_json::Value;

/// Get current configuration (sanitized, no API keys).
pub async fn get_config(State(state): State<AppState>) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let sanitized = serde_json::json!({
        "provider": core.config.provider,
        "model": core.config.model,
        "base_url": core.config.base_url,
        "execution_mode": core.config.execution_mode,
        "enabled_tools": core.config.enabled_tools,
        "mcp_servers": core.config.mcp_servers,
        "plugins_auto_discover": core.config.plugins_auto_discover,
    });
    super::ApiResponse::ok(sanitized)
}

/// Update default LLM configuration (provider, api_key, base_url, model).
pub async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let mut core = state.core.write().await;

    if let Some(p) = body.get("provider").and_then(|v| v.as_str()) {
        core.config.provider = p.parse().expect("invalid provider");
    }
    if let Some(k) = body.get("api_key").and_then(|v| v.as_str()) {
        core.config.api_key = k.to_string();
    }
    if let Some(u) = body.get("base_url").and_then(|v| v.as_str()) {
        core.config.base_url = u.to_string();
    }
    if let Some(m) = body.get("model").and_then(|v| v.as_str()) {
        core.config.model = m.to_string();
    }

    if let Err(e) = core.config.save() {
        return super::ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    let result = serde_json::json!({
        "status": "updated",
        "provider": core.config.provider,
        "base_url": core.config.base_url,
        "model": core.config.model,
    });
    super::ApiResponse::ok(result)
}

/// List all configured providers from the config.
pub async fn list_providers(State(state): State<AppState>) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let providers: Vec<Value> = core
        .config
        .providers
        .iter()
        .map(|(name, pc)| {
            serde_json::json!({
                "name": name,
                "provider": pc.provider,
                "model": pc.model,
                "base_url": pc.base_url,
            })
        })
        .collect();
    super::ApiResponse::ok(serde_json::json!({ "providers": providers }))
}
