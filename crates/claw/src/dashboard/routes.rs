use crate::dashboard::AppState;
use crate::llm::{LlmEvent, StreamResult};
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::Value;
use std::convert::Infallible;
use tokio::sync::mpsc;

// ── Response helpers ──

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data: Some(data),
            error: None,
        })
    }

    pub fn err(msg: &str) -> Json<Self> {
        Json(Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        })
    }
}

// ── Handlers ──

/// Health check endpoint.
pub async fn health() -> Json<ApiResponse<&'static str>> {
    ApiResponse::ok("OK")
}

/// Get current configuration (sanitized, no API keys).
pub async fn get_config(State(state): State<AppState>) -> Json<ApiResponse<Value>> {
    let core = state.core.lock().unwrap();
    let sanitized = serde_json::json!({
        "provider": core.config.provider,
        "model": core.config.model,
        "base_url": core.config.base_url,
        "enabled_tools": core.config.enabled_tools,
        "mcp_servers": core.config.mcp_servers,
        "plugins_auto_discover": core.config.plugins_auto_discover,
    });
    ApiResponse::ok(sanitized)
}

/// Send a message and start LLM processing.
/// Returns the session ID so the client can subscribe to SSE events.
pub async fn send_message(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<ApiResponse<Value>> {
    let text = match body.get("message").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return ApiResponse::err("Missing 'message' field"),
    };

    let mut core = state.core.lock().unwrap();

    // Create or get a session
    let session_id = core
        .session_mgr
        .current_id()
        .map(|id| id.to_string())
        .unwrap_or_default();

    if session_id.is_empty() {
        core.session_mgr.create_session();
    }

    let sid = core
        .session_mgr
        .current_id()
        .unwrap()
        .to_string();

    // Save user message
    core.session_mgr.append_message("user", &text, None);

    // Drop the lock before returning
    drop(core);

    ApiResponse::ok(serde_json::json!({
        "session_id": sid,
        "status": "processing",
    }))
}

/// Build API-compatible messages from JSONL session records.
fn build_dashboard_messages(core: &crate::core::AppCore, session_id: &str) -> Vec<Value> {
    let records = core.session_mgr.load_messages(session_id, 50);

    let mut msgs = vec![serde_json::json!({
        "role": "system",
        "content": crate::core::engine::build_system_prompt(
            &crate::tools::format_index(
                if core.config.enabled_tools.is_empty() { None } else { Some(&core.config.enabled_tools) }
            ),
            &core.cross_memory.format_hot_tools(&core.tool_cache),
            &core.skill_store.format_skills(),
            &core.cross_memory.format_user_memory(),
            &core.cross_memory.format_user_profile(),
        )
    })];

    for record in &records {
        let msg_type = record.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match msg_type {
            "user" | "assistant" => {
                if let Some(text) = record.get("text").and_then(|t| t.as_str()) {
                    msgs.push(serde_json::json!({
                        "role": msg_type,
                        "content": text
                    }));
                }
            }
            "tool_call" => {
                if let (Some(name), Some(args), Some(result)) = (
                    record.get("name").and_then(|n| n.as_str()),
                    record.get("args").and_then(|a| a.as_str()),
                    record.get("result").and_then(|r| r.as_str()),
                ) {
                    msgs.push(serde_json::json!({
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": record.get("name").and_then(|n| n.as_str()).unwrap_or(""),
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": args
                            }
                        }]
                    }));
                    msgs.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": name,
                        "content": result
                    }));
                }
            }
            _ => {}
        }
    }

    msgs
}

/// SSE stream for chat responses.
pub async fn chat_stream(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let (msgs, provider, tool_schemas, sid) = {
        let mut core = state.core.lock().unwrap();
        core.session_mgr.switch_to(&session_id);

        let msgs = build_dashboard_messages(&core, &session_id);
        let provider = crate::provider::create_provider(&core.config);
        let tool_schemas = {
            let enabled = if core.config.enabled_tools.is_empty() {
                None
            } else {
                Some(&core.config.enabled_tools)
            };
            crate::tools::ToolRegistry::new().enabled_schemas(enabled)
        };

        (msgs, provider, tool_schemas, session_id.clone())
    };

    let (tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    let save_state = state.clone();
    let save_sid = sid.clone();
    let save_msgs = msgs.clone();

    tokio::spawn(async move {
        match provider.stream_chat(&msgs, &tool_schemas, &tx).await {
            Ok(StreamResult::Text(usage, text)) => {
                let _ = tx.send(LlmEvent::Done(vec![], usage));

                // Save assistant response and update API cache
                let mut core = save_state.core.lock().unwrap();
                core.session_mgr.append_message("assistant", &text, None);
                let mut api_msgs = save_msgs;
                api_msgs.push(serde_json::json!({
                    "role": "assistant",
                    "content": text
                }));
                core.session_mgr.save_api_messages(&save_sid, &api_msgs);
            }
            Ok(StreamResult::ToolCalls(_tc, _reasoning)) => {
                // Tool calls not yet supported in dashboard chat
                let _ = tx.send(LlmEvent::Done(vec![], None));
            }
            Err(e) => {
                let _ = tx.send(LlmEvent::Error(format!("{}", e)));
            }
        }
    });

    let stream = futures_util::stream::unfold(Some(rx), |rx_opt| async {
        let mut rx = rx_opt?;
        loop {
            let event = rx.recv().await?;
            match event {
                LlmEvent::Token(t) => {
                    let sse = Event::default().event("token").data(t);
                    return Some((Ok::<_, Infallible>(sse), Some(rx)));
                }
                LlmEvent::Reasoning(t) => {
                    let sse = Event::default().event("reasoning").data(t);
                    return Some((Ok::<_, Infallible>(sse), Some(rx)));
                }
                LlmEvent::Status(s) => {
                    let sse = Event::default().event("status").data(s);
                    return Some((Ok::<_, Infallible>(sse), Some(rx)));
                }
                LlmEvent::Error(e) => {
                    let sse = Event::default().event("error").data(e);
                    return Some((Ok::<_, Infallible>(sse), None));
                }
                LlmEvent::Done(_, usage) => {
                    let data = serde_json::to_string(
                        &serde_json::json!({"usage": usage}),
                    )
                    .unwrap_or_default();
                    let sse = Event::default().event("done").data(data);
                    return Some((Ok::<_, Infallible>(sse), None));
                }
                _ => continue,
            }
        }
    });

    Sse::new(stream)
}

/// List all sessions.
pub async fn list_sessions(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Value>>> {
    let core = state.core.lock().unwrap();
    let sessions: Vec<Value> = core
        .session_mgr
        .sessions()
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "title": s.title,
                "message_count": s.message_count,
                "created_at": s.created_at,
            })
        })
        .collect();
    ApiResponse::ok(sessions)
}

/// Get session messages.
pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Value>> {
    let core = state.core.lock().unwrap();
    let messages = core.session_mgr.load_app_messages(&id, 100);
    let msgs: Vec<Value> = messages
        .iter()
        .map(|m| match m {
            crate::app::Message::User { text } => {
                serde_json::json!({"role": "user", "content": text})
            }
            crate::app::Message::Assistant { text } => {
                serde_json::json!({"role": "assistant", "content": text})
            }
            _ => serde_json::json!({"role": "unknown"}),
        })
        .collect();

    let meta = core.session_mgr.session_meta(&id);

    ApiResponse::ok(serde_json::json!({
        "id": id,
        "messages": msgs,
        "title": meta.as_ref().map(|m| &m.title),
    }))
}

/// Delete a session.
pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<&'static str>> {
    let mut core = state.core.lock().unwrap();
    core.session_mgr.delete_session(&id);
    drop(core);
    ApiResponse::ok("deleted")
}

/// List available tools.
pub async fn list_tools(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Value>>> {
    let core = state.core.lock().unwrap();
    let enabled = if core.config.enabled_tools.is_empty() {
        None
    } else {
        Some(&core.config.enabled_tools)
    };
    let schemas = crate::tools::ToolRegistry::new().enabled_schemas(enabled);
    ApiResponse::ok(schemas)
}

/// List installed plugins.
pub async fn list_plugins(
    State(_state): State<AppState>,
) -> Json<ApiResponse<Vec<Value>>> {
    let mgr = crate::plugin::PluginManager::new();
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
    ApiResponse::ok(plugins)
}
