use crate::server::AppState;
use axum::{
    Json,
    extract::State,
};
use serde_json::Value;
use std::sync::OnceLock;

static TOOL_REGISTRY: OnceLock<i_rs_claw_core::tools::ToolRegistry> = OnceLock::new();

/// List available tools.
pub async fn list_tools(State(state): State<AppState>) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    let enabled = if core.config.enabled_tools.is_empty() {
        None
    } else {
        Some(&core.config.enabled_tools)
    };
    let i_rs_tool_names: Vec<&str> = core.config.i_rs_tools.iter().map(|s| s.as_str()).collect();
    let reg = TOOL_REGISTRY.get_or_init(i_rs_claw_core::tools::ToolRegistry::new);
    let schemas = reg.enabled_schemas(&i_rs_tool_names, enabled);
    super::ApiResponse::ok(schemas)
}

/// List installed plugins.
pub async fn list_plugins(State(_state): State<AppState>) -> Json<super::ApiResponse<Vec<Value>>> {
    let mgr = i_rs_claw_core::plugin::PluginManager::new();
    let plugins: Vec<Value> = mgr
        .manifests
        .iter()
        .map(|m| {
            serde_json::json!({
                "name": m.plugin.name,
                "version": m.plugin.version,
                "description": m.plugin.description,
                "author": m.plugin.author,
                "enabled": mgr.is_enabled(&m.plugin.name),
            })
        })
        .collect();
    super::ApiResponse::ok(plugins)
}

/// List user-defined skills with parsed metadata.
pub async fn list_skills(
    State(state): State<AppState>,
) -> Json<super::ApiResponse<Vec<i_rs_claw_core::skill_store::SkillDefinition>>> {
    let core = state.core.read().await;
    let store = core.agent_store.skill_store_for("default", "default");
    let entries = store.list_skills();
    let skills: Vec<i_rs_claw_core::skill_store::SkillDefinition> = entries
        .iter()
        .map(|e| {
            let (fm, body) = i_rs_claw_core::skill_store::parse_frontmatter(&e.content);
            i_rs_claw_core::skill_store::SkillDefinition {
                name: e.name.clone(),
                description: fm
                    .as_ref()
                    .and_then(|t| {
                        t.get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| e.name.clone()),
                parameters: fm.as_ref().and_then(|t| {
                    t.get("parameters")
                        .and_then(|v| serde_json::to_value(v).ok())
                }),
                content: body.to_string(),
            }
        })
        .collect();
    super::ApiResponse::ok(skills)
}
