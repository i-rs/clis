use crate::server::AppState;
use crate::server::UserId;
use i_rs_claw_core::providers::ProviderKind;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::Value;

/// List available agent profiles.
pub async fn get_agents(State(state): State<AppState>) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    let agent_ids = core.config.all_agent_ids();
    let agents: Vec<Value> = agent_ids
        .iter()
        .map(|id| {
            let agent = core
                .config
                .agents
                .get(id)
                .or_else(|| core.config.sub_agents.get(id));
            let provider = agent
                .and_then(|a| a.provider)
                .unwrap_or(core.config.provider);
            let model = agent
                .and_then(|a| a.model.as_deref())
                .unwrap_or(&core.config.model);
            let base_url = agent
                .and_then(|a| a.base_url.as_deref())
                .unwrap_or(&core.config.base_url);
            let enabled_tools: &std::collections::HashSet<String> = agent
                .and_then(|a| a.enabled_tools.as_ref())
                .unwrap_or(&core.config.enabled_tools);
            let tools: Vec<&String> = enabled_tools.iter().collect();
            let is_sub = id != "default" && !core.config.agents.contains_key(id);
            let system_prompt = agent.and_then(|a| a.system_prompt.as_deref());
            let capabilities = agent.map(|a| &a.capabilities[..]).unwrap_or(&[]);
            let provider_ref = agent.and_then(|a| a.provider_ref.as_deref());
            serde_json::json!({
                "id": id,
                "provider": provider,
                "model": model,
                "base_url": base_url,
                "tool_count": enabled_tools.len(),
                "enabled_tools": tools,
                "system_prompt": system_prompt,
                "is_sub_agent": is_sub,
                "capabilities": capabilities,
                "provider_ref": provider_ref,
            })
        })
        .collect();
    super::ApiResponse::ok(agents)
}

/// Get detailed config for a single agent.
pub async fn get_agent_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let resolved = core.config.agent_config(&id);
    let raw_agent = core
        .config
        .agents
        .get(&id)
        .or_else(|| core.config.sub_agents.get(&id));
    let provider_ref = raw_agent.and_then(|a| a.provider_ref.as_deref());
    let tools: Vec<&String> = resolved.enabled_tools.iter().collect();
    super::ApiResponse::ok(serde_json::json!({
        "id": id,
        "provider": resolved.provider,
        "model": resolved.model,
        "base_url": resolved.base_url,
        "enabled_tools": tools,
        "tool_count": resolved.enabled_tools.len(),
        "system_prompt": resolved.system_prompt,
        "mcp_servers": resolved.mcp_servers,
        "allowed_dirs": resolved.allowed_dirs,
        "provider_ref": provider_ref,
    }))
}

/// Update an agent profile. Only provided fields are overridden.
pub async fn update_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    if id == "default" {
        return super::ApiResponse::err("Cannot update the default agent");
    }

    let mut core = state.core.write().await;

    // Get existing agent config
    let existing = match core.config.agents.get(&id) {
        Some(a) => a.clone(),
        None => return super::ApiResponse::err(&format!("Agent '{}' not found", id)),
    };

    // Merge body with existing (only override provided fields)
    let agent_config = i_rs_claw_core::config::AgentConfig {
        provider_ref: body
            .get("provider_ref")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(existing.provider_ref),
        provider: body
            .get("provider")
            .and_then(|v| v.as_str())
            .map(|s| s.parse::<ProviderKind>().unwrap_or(ProviderKind::OpenAI))
            .or(existing.provider),
        api_key: body
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(existing.api_key),
        base_url: body
            .get("base_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(existing.base_url),
        model: body
            .get("model")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(existing.model),
        enabled_tools: body
            .get("enabled_tools")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .or(existing.enabled_tools),
        system_prompt: body
            .get("system_prompt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or(existing.system_prompt),
        system_prompt_file: existing.system_prompt_file,
        mcp_servers: None, // inherit from existing via merge
        allowed_dirs: None,
        capabilities: existing.capabilities,
        execution_mode: existing.execution_mode,
    };

    core.config.agents.insert(id.clone(), agent_config);

    if let Err(e) = core.config.save() {
        return super::ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    super::ApiResponse::ok(serde_json::json!({
        "id": id,
        "status": "updated",
    }))
}

/// Create a new agent profile.
/// Body fields (all optional except `id`):
/// - `id`: agent identifier (required, cannot be "default")
/// - `provider`, `model`, `base_url`, `api_key`: provider overrides
/// - `system_prompt`: custom system prompt
/// - `enabled_tools`: list of tool names to enable (empty = all)
pub async fn create_agent(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let agent_id = match body.get("id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() && id != "default" => id.to_string(),
        Some("default") => return super::ApiResponse::err("Cannot create agent with id 'default'"),
        _ => return super::ApiResponse::err("Missing or invalid 'id' field"),
    };

    let mut core = state.core.write().await;

    // Check if agent already exists
    if core.config.agents.contains_key(&agent_id) {
        return super::ApiResponse::err(&format!("Agent '{}' already exists", agent_id));
    }

    // Build agent config from request body (all optional)
    let agent_config = i_rs_claw_core::config::AgentConfig {
        provider: body
            .get("provider")
            .and_then(|v| v.as_str())
            .map(|s| s.parse::<ProviderKind>().unwrap_or(ProviderKind::OpenAI)),
        api_key: body
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        base_url: body
            .get("base_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        model: body
            .get("model")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        enabled_tools: body
            .get("enabled_tools")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            }),
        system_prompt: body
            .get("system_prompt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        system_prompt_file: None,
        mcp_servers: None,
        allowed_dirs: None,
        capabilities: Vec::new(),
        provider_ref: body
            .get("provider_ref")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        execution_mode: None,
    };

    // Add to config (clone first so we can use fields for ConfigStore)
    let agent_config_clone = agent_config.clone();
    if let Err(e) = core.config.add_agent(&agent_id, agent_config) {
        return super::ApiResponse::err(&e.to_string());
    }

    // Create agent data directories on disk
    let claw_dir = match core.claw_dir() {
        Ok(d) => d,
        Err(e) => return super::ApiResponse::err(&format!("无法获取 claw 目录: {}", e)),
    };
    let agent_dir = claw_dir.join("agents").join(&agent_id);
    let _ = std::fs::create_dir_all(&agent_dir);

    // Initialize runtime data in agent_store
    let config = core.config.clone();
    core.agent_store.add_agent(&config, &agent_id);

    // Persist config
    if let Err(e) = core.config.save() {
        return super::ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    // Also persist to ConfigStore (DB backend)
    let now = chrono::Utc::now().timestamp();
    let row_provider_ref = agent_config_clone.provider_ref.clone();
    let row_provider = agent_config_clone.provider.map(|p| format!("{:?}", p)).unwrap_or_default();
    let row_api_key = agent_config_clone.api_key.clone().unwrap_or_default();
    let row_base_url = agent_config_clone.base_url.clone().unwrap_or_default();
    let row_model = agent_config_clone.model.clone().unwrap_or_default();
    let row_tools: Vec<String> = agent_config_clone.enabled_tools.clone().unwrap_or_default().into_iter().collect();
    let row_prompt = agent_config_clone.system_prompt.clone().unwrap_or_default();
    let row_prompt_file = agent_config_clone.system_prompt_file.clone();
    let row_caps = agent_config_clone.capabilities.clone();
    let row_execution = agent_config_clone.execution_mode.map(|e| format!("{:?}", e)).unwrap_or_else(|| "React".into());
    let _ = core.config_store.agent_configs.upsert(&i_rs_claw_core::storage::config_store::AgentConfigRow {
        user_id: user_id.into(),
        agent_id: agent_id.clone(),
        provider_ref: row_provider_ref,
        provider: row_provider,
        api_key: row_api_key,
        base_url: row_base_url,
        model: row_model,
        enabled_tools: row_tools,
        system_prompt: row_prompt,
        system_prompt_file: row_prompt_file,
        capabilities: row_caps,
        execution_mode: row_execution,
        created_at: now,
        updated_at: now,
    }).await;

    super::ApiResponse::ok(serde_json::json!({
        "id": agent_id,
        "status": "created",
    }))
}

/// Delete an agent profile. Cannot delete "default".
pub async fn delete_agent(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    if id == "default" {
        return super::ApiResponse::err("Cannot delete the default agent");
    }

    let mut core = state.core.write().await;

    // Remove from config
    if let Err(e) = core.config.remove_agent(&id) {
        return super::ApiResponse::err(&e.to_string());
    }

    // Remove from runtime store
    core.agent_store.remove_agent(&id);

    // Persist config
    if let Err(e) = core.config.save() {
        return super::ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    // Also delete from ConfigStore
    let _ = core.config_store.agent_configs.delete(&user_id, &id).await;

    super::ApiResponse::ok(serde_json::json!({
        "id": id,
        "status": "deleted",
    }))
}
