use crate::server::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::Value;

/// POST /api/providers — Create a new provider configuration.
pub async fn create_provider(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let name = match body.get("name").and_then(|v| v.as_str()) {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => return super::ApiResponse::err("Missing or invalid 'name' field"),
    };

    let provider_str = body
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("openai");
    let provider = match provider_str.parse::<i_rs_claw_core::providers::ProviderKind>() {
        Ok(p) => p,
        Err(_) => return super::ApiResponse::err(&format!("Unknown provider: {}", provider_str)),
    };
    let api_key = body.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let base_url = body.get("base_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut core = state.core.write().await;

    // Persist to config.toml so chat_loop can use it immediately
    core.config.providers.insert(
        name.clone(),
        i_rs_claw_core::config::ProviderConfig {
            provider,
            api_key: api_key.clone(),
            base_url: base_url.clone(),
            model: model.clone(),
        },
    );
    if let Err(e) = core.config.save() {
        return super::ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    // Also persist to ConfigStore (DB backend)
    let now = chrono::Utc::now().timestamp();
    let row = i_rs_claw_core::storage::config_store::ProviderConfigRow {
        name: name.clone(),
        provider: provider_str.to_string(),
        api_key,
        base_url,
        model,
        created_at: now,
        updated_at: now,
    };

    match core.config_store.provider_configs.upsert(&row).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "name": name,
            "status": "created",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to create provider: {}", e)),
    }
}

/// PUT /api/providers/{name} — Update an existing provider configuration.
pub async fn update_provider(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let mut core = state.core.write().await;

    // Get existing provider from config.toml
    let existing = core.config.providers.get(&name).cloned();

    let provider_str = body
        .get("provider")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| existing.as_ref().map(|p| format!("{:?}", p.provider)))
        .unwrap_or_else(|| "openai".to_string());

    let api_key = body
        .get("api_key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| existing.as_ref().map(|p| p.api_key.clone()).unwrap_or_default());

    let base_url = body
        .get("base_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| existing.as_ref().map(|p| p.base_url.clone()).unwrap_or_default());

    let model = body
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| existing.as_ref().map(|p| p.model.clone()).unwrap_or_default());

    // Persist to config.toml so chat_loop sees the update immediately
    let provider_parsed = provider_str.parse::<i_rs_claw_core::providers::ProviderKind>().unwrap_or(i_rs_claw_core::providers::ProviderKind::OpenAI);
    core.config.providers.insert(
        name.clone(),
        i_rs_claw_core::config::ProviderConfig {
            provider: provider_parsed,
            api_key: api_key.clone(),
            base_url: base_url.clone(),
            model: model.clone(),
        },
    );
    if let Err(e) = core.config.save() {
        return super::ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    // Also persist to ConfigStore
    let now = chrono::Utc::now().timestamp();
    let row = i_rs_claw_core::storage::config_store::ProviderConfigRow {
        name: name.clone(),
        provider: provider_str,
        api_key,
        base_url,
        model,
        created_at: now,
        updated_at: now,
    };
    match core.config_store.provider_configs.upsert(&row).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "name": name,
            "status": "updated",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to update provider: {}", e)),
    }
}

/// DELETE /api/providers/{name} — Delete a provider configuration.
pub async fn delete_provider(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    let mut core = state.core.write().await;

    if !core.config.providers.contains_key(&name) {
        return super::ApiResponse::err(&format!("Provider '{}' not found", name));
    }

    core.config.providers.remove(&name);

    if let Err(e) = core.config.save() {
        return super::ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    if let Err(e) = core.config_store.provider_configs.delete(&name).await {
        tracing::error!(%e, "DB cleanup failed for provider delete");
    }

    super::ApiResponse::ok(serde_json::json!({
        "name": name,
        "status": "deleted",
    }))
}
