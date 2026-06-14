use crate::server::AppState;
use crate::server::UserId;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct McpListQuery {
    pub agent_id: Option<String>,
}

/// GET /api/mcp — List MCP server configs. Query: ?agent_id=xxx
pub async fn list_mcp_configs(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Query(query): Query<McpListQuery>,
) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    match core
        .config_store
        .mcp_servers
        .load_for(&user_id, query.agent_id.as_deref())
        .await
    {
        Ok(rows) => {
            let result: Vec<Value> = rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "agent_id": r.agent_id,
                        "transport_type": r.transport_type,
                        "command": r.command,
                        "args_json": r.args_json,
                        "url": r.url,
                        "env_json": r.env_json,
                        "enabled": r.enabled,
                    })
                })
                .collect();
            super::ApiResponse::ok(result)
        }
        Err(e) => super::ApiResponse::err(&format!("Failed to list MCP configs: {}", e)),
    }
}

/// POST /api/mcp — Create a new MCP server config.
pub async fn create_mcp_config(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let name = match body.get("name").and_then(|v| v.as_str()) {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => return super::ApiResponse::err("Missing or invalid 'name' field"),
    };

    let core = state.core.read().await;
    let row = i_rs_claw_core::storage::config_store::McpServerConfigRow {
        user_id,
        agent_id: body.get("agent_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
        name,
        transport_type: body
            .get("transport_type")
            .and_then(|v| v.as_str())
            .unwrap_or("stdio")
            .to_string(),
        command: body.get("command").and_then(|v| v.as_str()).map(|s| s.to_string()),
        args_json: body.get("args_json").and_then(|v| v.as_str()).map(|s| s.to_string()),
        url: body.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
        env_json: body.get("env_json").and_then(|v| v.as_str()).map(|s| s.to_string()),
        enabled: body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
    };

    match core.config_store.mcp_servers.upsert(&row).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "name": row.name,
            "status": "created",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to create MCP config: {}", e)),
    }
}

/// PUT /api/mcp/{name} — Update an MCP server config. Query: ?agent_id=xxx
pub async fn update_mcp_config(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(name): Path<String>,
    Query(query): Query<McpListQuery>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;

    // Load existing row to merge with — prevents accidental field resets
    let existing = core.config_store.mcp_servers
        .load_for(&user_id, query.agent_id.as_deref())
        .await
        .ok()
        .and_then(|rows| rows.into_iter().find(|r| r.name == name));

    let row = i_rs_claw_core::storage::config_store::McpServerConfigRow {
        user_id,
        agent_id: query.agent_id.or_else(|| {
            body.get("agent_id").and_then(|v| v.as_str()).map(|s| s.to_string())
        }).or(existing.as_ref().and_then(|e| e.agent_id.clone())),
        name,
        transport_type: body
            .get("transport_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| existing.as_ref().map(|e| e.transport_type.clone()))
            .unwrap_or_else(|| "stdio".to_string()),
        command: body.get("command").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| existing.as_ref().and_then(|e| e.command.clone())),
        args_json: body.get("args_json").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| existing.as_ref().and_then(|e| e.args_json.clone())),
        url: body.get("url").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| existing.as_ref().and_then(|e| e.url.clone())),
        env_json: body.get("env_json").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| existing.as_ref().and_then(|e| e.env_json.clone())),
        enabled: body.get("enabled").and_then(|v| v.as_bool())
            .or_else(|| existing.as_ref().map(|e| e.enabled))
            .unwrap_or(true),
    };

    match core.config_store.mcp_servers.upsert(&row).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "name": row.name,
            "status": "updated",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to update MCP config: {}", e)),
    }
}

/// DELETE /api/mcp/{name} — Delete an MCP server config. Query: ?agent_id=xxx
pub async fn delete_mcp_config(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(name): Path<String>,
    Query(query): Query<McpListQuery>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    match core
        .config_store
        .mcp_servers
        .delete(&user_id, query.agent_id.as_deref(), &name)
        .await
    {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "name": name,
            "status": "deleted",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to delete MCP config: {}", e)),
    }
}
