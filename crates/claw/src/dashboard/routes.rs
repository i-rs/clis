use crate::dashboard::AppState;
use crate::llm::LlmEvent;
use crate::stats::StatsPeriod;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
    response::sse::{Event, Sse},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use std::sync::OnceLock;
use tokio::sync::mpsc;

static TOOL_REGISTRY: OnceLock<crate::tools::ToolRegistry> = OnceLock::new();

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

/// Convert an app::Message to an API-compatible JSON value.
fn message_to_api_json(msg: &crate::app::Message) -> Value {
    match msg {
        crate::app::Message::User { text } => {
            serde_json::json!({"role": "user", "content": text})
        }
        crate::app::Message::Assistant { text, reasoning } => {
            let mut msg = serde_json::json!({"role": "assistant", "content": text});
            if !reasoning.is_empty() {
                msg["reasoning"] = Value::String(reasoning.clone());
            }
            msg
        }
        crate::app::Message::ToolCall {
            name, args, result, ..
        } => {
            serde_json::json!({"role": "tool_call", "name": name, "args": args, "result": result})
        }
        crate::app::Message::Image { path, alt_text, width, height, format } => {
            serde_json::json!({
                "role": "image",
                "path": path,
                "alt_text": alt_text,
                "width": width,
                "height": height,
                "format": format,
                "url": format!("/api/images/{}", path)
            })
        }
        _ => serde_json::json!({"role": "unknown"}),
    }
}

/// Health check endpoint.
pub async fn health() -> Json<ApiResponse<&'static str>> {
    ApiResponse::ok("OK")
}

#[derive(Deserialize)]
pub struct StatsQuery {
    #[serde(default = "default_period")]
    pub period: String,
}

fn default_period() -> String {
    "today".to_string()
}

/// Get token usage statistics.
/// Query params: ?period=today|7d|30d|all (default: today)
pub async fn get_stats(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
) -> Json<ApiResponse<Value>> {
    let core = state.core.read().await;

    if !core.config.stats.enabled {
        return ApiResponse::err("Token usage statistics are disabled");
    }

    let period = match query.period.as_str() {
        "7d" | "7days" => StatsPeriod::Last7Days,
        "30d" | "30days" => StatsPeriod::Last30Days,
        "all" => StatsPeriod::All,
        _ => StatsPeriod::Today,
    };

    let result = core.stats_manager.query(period);
    let today = core.stats_manager.today_summary();

    match serde_json::to_value(&result) {
        Ok(mut v) => {
            if let Some(obj) = v.as_object_mut() {
                obj.insert(
                    "today".to_string(),
                    serde_json::json!({
                        "requests": today.requests,
                        "tokens": today.tokens,
                        "cost_usd": today.cost_usd,
                    }),
                );
            }
            ApiResponse::ok(v)
        }
        Err(e) => ApiResponse::err(&format!("Failed to serialize stats: {}", e)),
    }
}

/// Get current configuration (sanitized, no API keys).
pub async fn get_config(State(state): State<AppState>) -> Json<ApiResponse<Value>> {
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
    ApiResponse::ok(sanitized)
}

/// Update default LLM configuration (provider, api_key, base_url, model).
pub async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<ApiResponse<Value>> {
    let mut core = state.core.write().await;

    if let Some(p) = body.get("provider").and_then(|v| v.as_str()) {
        core.config.provider = p.to_string();
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
        return ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    let result = serde_json::json!({
        "status": "updated",
        "provider": core.config.provider,
        "base_url": core.config.base_url,
        "model": core.config.model,
    });
    ApiResponse::ok(result)
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

    let agent_id = body
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let mut core = state.core.write().await;

    // Create or get a session
    let session_id = core
        .session_mgr
        .current_id()
        .map(|id| id.to_string())
        .unwrap_or_default();

    if session_id.is_empty() {
        core.session_mgr.create_session_for(&agent_id);
    }

    let sid = match core.session_mgr.current_id() {
        Some(id) => id.to_string(),
        None => return ApiResponse::err("没有活跃会话"),
    };

    // Save user message
    core.session_mgr.append_message("user", &text, None);

    {
        let layered = core.agent_store.layered_memory_for_mut(&agent_id);
        layered.record_user_statement(&text);
    }

    // Drop the write lock before spawning LLM
    drop(core);

    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmEvent>();
    {
        let core = state.core.read().await;
        let records = core.session_mgr.load_messages(&sid, 50);
        let msgs = core.build_messages_from_jsonl(&records, &agent_id);
        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &records);
    }

    let bg_state = state.clone();
    let bg_sid = sid.clone();
    tokio::spawn(async move {
        while let Some(event) = llm_rx.recv().await {
            match &event {
                LlmEvent::Done(msgs, _usage, _trace_id) => {
                    let mut core = bg_state.core.write().await;
                    if let Some(last) = msgs.last()
                        && last.get("role").and_then(|r| r.as_str()) == Some("assistant")
                    {
                        let text = last.get("content").and_then(|c| c.as_str()).unwrap_or("");
                        let reasoning = last
                            .get("reasoning_content")
                            .and_then(|r| r.as_str())
                            .unwrap_or("");
                        let extra = if !reasoning.is_empty() {
                            Some(serde_json::json!({"reasoning": reasoning}))
                        } else {
                            None
                        };
                        if !text.is_empty() || extra.is_some() {
                            core.session_mgr.append_message("assistant", text, extra);
                        }
                    }
                    crate::core::save_chat_result(&mut core.session_mgr, &bg_sid, &msgs);
                    break;
                }
                LlmEvent::Error(e) => {
                    let mut core = bg_state.core.write().await;
                    core.session_mgr.mark_error(&bg_sid, e);
                    break;
                }
                _ => {}
            }
        }
    });

    ApiResponse::ok(serde_json::json!({
        "session_id": sid,
        "status": "processing",
    }))
}

/// SSE stream for chat responses.
pub async fn chat_stream(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> axum::response::Response {
    let (llm_tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    {
        let mut core = state.core.write().await;
        core.session_mgr.switch_to(&session_id);

        let agent_id = core
            .session_mgr
            .session_meta(&session_id)
            .map(|m| m.agent_id.clone())
            .unwrap_or_else(|| "default".to_string());

        let records = core.session_mgr.load_messages(&session_id, 50);
        let msgs = core.build_messages_from_jsonl(&records, &agent_id);

        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &records);
    }

    let stream_state = state.clone();
    let stream_sid = session_id.clone();

    let stream = futures_util::stream::unfold(
        (Some(rx), stream_state, stream_sid),
        |(rx_opt, state, sid)| async move {
            let mut rx = rx_opt?;
            loop {
                let event = rx.recv().await?;
                match event {
                    LlmEvent::ToolExecuted {
                        name,
                        args,
                        result,
                        step,
                        total_steps,
                    } => {
                        let mut core = state.core.write().await;
                        let i_rs_index = core.config.i_rs_tool_index.clone();
                        let agent_id = core
                            .session_mgr
                            .session_meta(&sid)
                            .map(|m| m.agent_id.clone())
                            .unwrap_or_else(|| "default".to_string());
                        crate::core::record_tool_memory(
                            &mut core.agent_store,
                            &i_rs_index,
                            &agent_id,
                            &name,
                            &args,
                            &result,
                        );
                        if !result.starts_with("错误") && !result.starts_with("护栏拦截") {
                            crate::core::record_layered_tool_memory(
                                &mut core.agent_store,
                                &agent_id,
                                &name,
                                &result,
                            );
                        }
                        drop(core);

                        let data = serde_json::to_string(&serde_json::json!({
                            "name": name, "args": args, "result": result,
                            "step": step, "total_steps": total_steps,
                        }))
                        .unwrap_or_default();
                        let sse = Event::default().event("tool_executed").data(data);
                        return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid)));
                    }
                    LlmEvent::Done(msgs, usage, _trace_id) => {
                        let mut core = state.core.write().await;
                        let agent_id = core
                            .session_mgr
                            .session_meta(&sid)
                            .map(|m| m.agent_id.clone())
                            .unwrap_or_else(|| "default".to_string());
                        if let Some(last) = msgs.last()
                            && last.get("role").and_then(|r| r.as_str()) == Some("assistant")
                        {
                            let text = last.get("content").and_then(|c| c.as_str()).unwrap_or("");
                            let reasoning = last
                                .get("reasoning_content")
                                .and_then(|r| r.as_str())
                                .unwrap_or("");
                            let extra = if !reasoning.is_empty() {
                                Some(serde_json::json!({"reasoning": reasoning}))
                            } else {
                                None
                            };
                            if !text.is_empty() || extra.is_some() {
                                core.session_mgr.append_message("assistant", text, extra);
                            }
                        }
                        crate::core::save_chat_result(&mut core.session_mgr, &sid, &msgs);
                        let _quality = core.evaluate_completed_session(&sid);
                        core.agent_store.memory_for_mut(&agent_id).flush();
                        drop(core);

                        let data = serde_json::to_string(&serde_json::json!({"usage": usage}))
                            .unwrap_or_default();
                        let sse = Event::default().event("done").data(data);
                        return Some((Ok::<_, Infallible>(sse), (None, state, sid)));
                    }
                    LlmEvent::Error(e) => {
                        let mut core = state.core.write().await;
                        core.session_mgr.mark_error(&sid, &e);
                        drop(core);

                        let sse = Event::default().event("error").data(e);
                        return Some((Ok::<_, Infallible>(sse), (None, state, sid)));
                    }
                    LlmEvent::Token(t) => {
                        let sse = Event::default().event("token").data(t);
                        return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid)));
                    }
                    LlmEvent::Reasoning(t) => {
                        let sse = Event::default().event("reasoning").data(t);
                        return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid)));
                    }
                    LlmEvent::Status(s) => {
                        let sse = Event::default().event("status").data(s);
                        return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid)));
                    }
                    LlmEvent::NewRound => {
                        let sse = Event::default().event("new_round").data("");
                        return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid)));
                    }
                    LlmEvent::ImageGenerated { path, alt_text, format, width, height } => {
                        let data = serde_json::to_string(&serde_json::json!({
                            "path": path, "alt_text": alt_text,
                            "format": format, "width": width, "height": height,
                        }))
                        .unwrap_or_default();
                        let sse = Event::default().event("image_generated").data(data);
                        return Some((Ok::<_, Infallible>(sse), (Some(rx), state, sid)));
                    }
                    _ => continue,
                }
            }
        },
    );

    Sse::new(stream).into_response()
}

/// Get current session info.
pub async fn get_current_session(State(state): State<AppState>) -> Json<ApiResponse<Value>> {
    let core = state.core.read().await;
    let id = core.session_mgr.current_id().map(|s| s.to_string());
    match id {
        Some(ref sid) => {
            let meta = core.session_mgr.session_meta(sid);
            let messages = core.session_mgr.load_app_messages(sid, 50);
            let msgs: Vec<Value> = messages.iter().map(message_to_api_json).collect();
            ApiResponse::ok(serde_json::json!({
                "id": sid,
                "title": meta.as_ref().map(|m| &m.title),
                "message_count": meta.as_ref().map(|m| m.message_count).unwrap_or(0),
                "messages": msgs,
                "agent_id": meta.as_ref().map(|m| &m.agent_id),
            }))
        }
        None => ApiResponse::ok(serde_json::json!({
            "id": null,
            "title": null,
            "message_count": 0,
            "messages": [],
        })),
    }
}

/// Create a new session and switch to it.
pub async fn create_session(
    State(state): State<AppState>,
    body: Option<Json<Value>>,
) -> Json<ApiResponse<Value>> {
    let agent_id = body
        .as_ref()
        .and_then(|b| b.get("agent_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    let mut core = state.core.write().await;
    let id = core.session_mgr.create_session_for(agent_id);
    ApiResponse::ok(serde_json::json!({
        "id": id,
        "title": "",
        "message_count": 0,
        "agent_id": agent_id,
    }))
}

/// Switch to an existing session.
pub async fn switch_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Value>> {
    let mut core = state.core.write().await;
    if core.session_mgr.switch_to(&id) {
        let meta = core.session_mgr.session_meta(&id);
        ApiResponse::ok(serde_json::json!({
            "id": id,
            "title": meta.as_ref().map(|m| &m.title),
            "message_count": meta.as_ref().map(|m| m.message_count).unwrap_or(0),
            "agent_id": meta.as_ref().map(|m| &m.agent_id),
        }))
    } else {
        ApiResponse::err("Session not found")
    }
}

/// List all sessions.
pub async fn list_sessions(State(state): State<AppState>) -> Json<ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
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
                "agent_id": s.agent_id,
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
    let core = state.core.read().await;
    let messages = core.session_mgr.load_app_messages(&id, 100);
    let msgs: Vec<Value> = messages.iter().map(message_to_api_json).collect();

    let meta = core.session_mgr.session_meta(&id);

    ApiResponse::ok(serde_json::json!({
        "id": id,
        "messages": msgs,
        "title": meta.as_ref().map(|m| &m.title),
        "agent_id": meta.as_ref().map(|m| &m.agent_id),
    }))
}

/// Delete a session.
pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<&'static str>> {
    let mut core = state.core.write().await;
    let agent_id = core
        .session_mgr
        .session_meta(&id)
        .map(|m| m.agent_id.clone())
        .unwrap_or_else(|| "default".to_string());
    core.session_mgr.delete_session(&id);
    {
        let layered = core.agent_store.layered_memory_for_mut(&agent_id);
        layered.end_session();
    }
    drop(core);
    ApiResponse::ok("deleted")
}

/// POST /api/sessions/{id}/feedback — record user feedback (thumbs up/down).
pub async fn post_session_feedback(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Json<ApiResponse<&'static str>> {
    let positive = body
        .get("positive")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let feedback_msg = body.get("message").and_then(|v| v.as_str());

    let mut core = state.core.write().await;
    let agent_id = core
        .session_mgr
        .session_meta(&id)
        .map(|m| m.agent_id.clone())
        .unwrap_or_else(|| "default".to_string());

    // Record in cross-session memory
    core.agent_store
        .memory_for_mut(&agent_id)
        .record_session_feedback(&id, positive);

    // Append feedback to session
    core.session_mgr.append_message(
        "feedback",
        &format!("positive: {}", positive),
        Some(serde_json::json!({
            "positive": positive,
            "message": feedback_msg,
        })),
    );

    core.agent_store.memory_for_mut(&agent_id).flush();
    drop(core);

    ApiResponse::ok("ok")
}

/// List available agent profiles.
pub async fn get_agents(State(state): State<AppState>) -> Json<ApiResponse<Vec<Value>>> {
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
                .and_then(|a| a.provider.as_deref())
                .unwrap_or(&core.config.provider);
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
            })
        })
        .collect();
    ApiResponse::ok(agents)
}

/// Get detailed config for a single agent.
pub async fn get_agent_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Value>> {
    let core = state.core.read().await;
    let resolved = core.config.agent_config(&id);
    let tools: Vec<&String> = resolved.enabled_tools.iter().collect();
    ApiResponse::ok(serde_json::json!({
        "id": id,
        "provider": resolved.provider,
        "model": resolved.model,
        "base_url": resolved.base_url,
        "enabled_tools": tools,
        "tool_count": resolved.enabled_tools.len(),
        "system_prompt": resolved.system_prompt,
        "mcp_servers": resolved.mcp_servers,
        "allowed_dirs": resolved.allowed_dirs,
    }))
}

/// Update an agent profile. Only provided fields are overridden.
pub async fn update_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Json<ApiResponse<Value>> {
    if id == "default" {
        return ApiResponse::err("Cannot update the default agent");
    }

    let mut core = state.core.write().await;

    // Get existing agent config
    let existing = match core.config.agents.get(&id) {
        Some(a) => a.clone(),
        None => return ApiResponse::err(&format!("Agent '{}' not found", id)),
    };

    // Merge body with existing (only override provided fields)
    let agent_config = crate::config::AgentConfig {
        provider: body
            .get("provider")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
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
        return ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    ApiResponse::ok(serde_json::json!({
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
    Json(body): Json<Value>,
) -> Json<ApiResponse<Value>> {
    let agent_id = match body.get("id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() && id != "default" => id.to_string(),
        Some("default") => return ApiResponse::err("Cannot create agent with id 'default'"),
        _ => return ApiResponse::err("Missing or invalid 'id' field"),
    };

    let mut core = state.core.write().await;

    // Check if agent already exists
    if core.config.agents.contains_key(&agent_id) {
        return ApiResponse::err(&format!("Agent '{}' already exists", agent_id));
    }

    // Build agent config from request body (all optional)
    let agent_config = crate::config::AgentConfig {
        provider: body
            .get("provider")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
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
        execution_mode: None,
    };

    // Add to config
    if let Err(e) = core.config.add_agent(&agent_id, agent_config) {
        return ApiResponse::err(&e.to_string());
    }

    // Create agent data directories on disk
    let claw_dir = match core.claw_dir() {
        Ok(d) => d,
        Err(e) => return ApiResponse::err(&format!("无法获取 claw 目录: {}", e)),
    };
    let agent_dir = claw_dir.join("agents").join(&agent_id);
    let _ = std::fs::create_dir_all(&agent_dir);

    // Initialize runtime data in agent_store
    let config = core.config.clone();
    core.agent_store.add_agent(&config, &agent_id);

    // Persist config
    if let Err(e) = core.config.save() {
        return ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    ApiResponse::ok(serde_json::json!({
        "id": agent_id,
        "status": "created",
    }))
}

/// Delete an agent profile. Cannot delete "default".
pub async fn delete_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Value>> {
    if id == "default" {
        return ApiResponse::err("Cannot delete the default agent");
    }

    let mut core = state.core.write().await;

    // Remove from config
    if let Err(e) = core.config.remove_agent(&id) {
        return ApiResponse::err(&e.to_string());
    }

    // Remove from runtime store
    core.agent_store.remove_agent(&id);

    // Persist config
    if let Err(e) = core.config.save() {
        return ApiResponse::err(&format!("Failed to save config: {}", e));
    }

    ApiResponse::ok(serde_json::json!({
        "id": id,
        "status": "deleted",
    }))
}

/// List available tools.
pub async fn list_tools(State(state): State<AppState>) -> Json<ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    let enabled = if core.config.enabled_tools.is_empty() {
        None
    } else {
        Some(&core.config.enabled_tools)
    };
    let i_rs_tool_names: Vec<&str> = core.config.i_rs_tools.iter().map(|s| s.as_str()).collect();
    let reg = TOOL_REGISTRY.get_or_init(crate::tools::ToolRegistry::new);
    let schemas = reg.enabled_schemas(&i_rs_tool_names, enabled);
    ApiResponse::ok(schemas)
}

/// List installed plugins.
pub async fn list_plugins(State(_state): State<AppState>) -> Json<ApiResponse<Vec<Value>>> {
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

/// List user-defined skills with parsed metadata.
pub async fn list_skills(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<crate::skill_store::SkillDefinition>>> {
    let core = state.core.read().await;
    let store = core.agent_store.skill_store_for("default");
    let entries = store.list_skills();
    let skills: Vec<crate::skill_store::SkillDefinition> = entries
        .iter()
        .map(|e| {
            let (fm, body) = crate::skill_store::parse_frontmatter(&e.content);
            crate::skill_store::SkillDefinition {
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
    ApiResponse::ok(skills)
}

/// Serve generated images from ~/.i-rs/claw/images/.
pub async fn serve_image(
    Path(filename): Path<String>,
) -> axum::response::Response {
    use axum::body::Body;
    use axum::http::{StatusCode, header};

    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Invalid filename"))
            .expect("serve_image response builder");
    }

    let claw_dir = match crate::utils::claw_dir() {
        Some(d) => d,
        None => {
            return axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Cannot resolve claw directory"))
                .expect("serve_image response builder");
        }
    };

    let filepath = claw_dir.join("images").join(&filename);

    match std::fs::read(&filepath) {
        Ok(content) => {
            let mime = mime_guess::from_path(&filepath).first_or_octet_stream();
            axum::response::Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content))
                .expect("serve_image response builder")
        }
        Err(_) => axum::response::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Image not found"))
            .expect("serve_image response builder"),
    }
}

// ── Guardrails ──

pub async fn check_guardrails(
    Json(body): Json<Value>,
) -> Json<ApiResponse<Value>> {
    let input = body.get("input").and_then(|v| v.as_str());
    let output = body.get("output").and_then(|v| v.as_str());
    let tool_name = body.get("tool_name").and_then(|v| v.as_str());
    let tool_args = body.get("tool_args");

    let mut results = Vec::new();

    let mgr = crate::tools::guardrails::GuardrailManager::new();

    if let Some(text) = input {
        let r = mgr.check_input(text).await;
        results.push(serde_json::json!({
            "type": "input",
            "allowed": r.allowed,
            "reason": r.reason,
        }));
    }

    if let Some(text) = output {
        let r = mgr.check_output(text).await;
        results.push(serde_json::json!({
            "type": "output",
            "allowed": r.allowed,
            "reason": r.reason,
        }));
    }

    if let (Some(name), Some(args)) = (tool_name, tool_args) {
        let r = mgr.check_tool_call(name, args).await;
        results.push(serde_json::json!({
            "type": "tool_call",
            "allowed": r.allowed,
            "reason": r.reason,
        }));
    }

    if results.is_empty() {
        return ApiResponse::err("需要 input, output 或 tool_name+tool_args 参数");
    }

    let all_allowed = results.iter().all(|r| r["allowed"].as_bool().unwrap_or(false));
    ApiResponse::ok(serde_json::json!({
        "allowed": all_allowed,
        "checks": results,
    }))
}

// ── Checkpoints ──

pub async fn list_checkpoints(
    State(state): State<AppState>,
    Query(query): Query<CheckpointQuery>,
) -> Json<ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    let checkpoints: Vec<Value> = if let Ok(store) = core.checkpoint_store.lock() {
        store.list().iter().filter(|(id, _, _)| {
            query
                .session_id
                .as_ref()
                .map_or(true, |sid| id.starts_with(&format!("cp_{}_", sid)))
        }).map(|(id, round, ts)| {
            serde_json::json!({
                "id": id,
                "round": round,
                "timestamp": ts,
            })
        }).collect()
    } else {
        Vec::new()
    };
    drop(core);
    ApiResponse::ok(checkpoints)
}

#[derive(Deserialize)]
pub struct CheckpointQuery {
    pub session_id: Option<String>,
}

// ── Checkpoint Detail & Restore ──

pub async fn get_checkpoint_detail(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Json<ApiResponse<Value>> {
    let core = state.core.read().await;
    let result = {
        let Ok(store) = core.checkpoint_store.lock() else {
            drop(core);
            return ApiResponse::err("检查点存储不可用");
        };
        let Some(cp) = store.get(&id) else {
            drop(store);
            drop(core);
            return ApiResponse::err("检查点未找到");
        };
        let data = serde_json::json!({
            "id": cp.id,
            "session_id": cp.session_id,
            "round": cp.round,
            "timestamp": cp.timestamp,
            "message_count": cp.messages.len(),
            "tool_result_count": cp.tool_results.len(),
            "messages": cp.messages,
            "tool_results": cp.tool_results,
        });
        drop(store);
        data
    };
    drop(core);
    ApiResponse::ok(result)
}

pub async fn restore_checkpoint(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<ApiResponse<Value>> {
    let session_id = body
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let round = body
        .get("round")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    if session_id.is_empty() {
        return ApiResponse::err("需要 session_id 参数");
    }
    let cp_id = format!("cp_{}_{}", session_id, round);
    let core = state.core.read().await;
    let result = {
        let Ok(store) = core.checkpoint_store.lock() else {
            drop(core);
            return ApiResponse::err("检查点存储不可用");
        };
        let Some(cp) = store.get(&cp_id) else {
            drop(store);
            drop(core);
            return ApiResponse::err(&format!(
                "未找到 session={} round={} 的检查点",
                session_id, round
            ));
        };
        let messages = cp.restore_messages();
        let data = serde_json::json!({
            "restored": true,
            "session_id": cp.session_id,
            "round": cp.round,
            "message_count": messages.len(),
        });
        drop(store);
        data
    };
    drop(core);
    ApiResponse::ok(result)
}

// ── Layered Memory ──

pub async fn get_layered_memory(
    State(state): State<AppState>,
    Query(query): Query<MemoryQuery>,
) -> Json<ApiResponse<Value>> {
    let core = state.core.read().await;
    let agent_id = query.agent_id.as_deref().unwrap_or("default");
    let layered = core.agent_store.layered_memory_for(agent_id);
    let summary = layered.format_for_prompt();
    let fact_count = layered.long_term.facts.len();
    let entity_count = layered.working.entities.len();
    drop(core);
    ApiResponse::ok(serde_json::json!({
        "agent_id": agent_id,
        "summary": summary,
        "document_count": fact_count,
        "entity_count": entity_count,
    }))
}

pub async fn clear_layered_memory(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<ApiResponse<&'static str>> {
    let mut core = state.core.write().await;
    let agent_id = body
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let layered = core.agent_store.layered_memory_for_mut(agent_id);
    layered.working.clear();
    layered.long_term.facts.clear();
    layered.summaries.clear();
    drop(core);
    ApiResponse::ok("cleared")
}

#[derive(Deserialize)]
pub struct MemoryQuery {
    pub agent_id: Option<String>,
}

pub async fn search_layered_memory(
    State(state): State<AppState>,
    Query(query): Query<MemorySearchQuery>,
) -> Json<ApiResponse<Value>> {
    let core = state.core.read().await;
    let agent_id = query.agent_id.as_deref().unwrap_or("default");
    let layered = core.agent_store.layered_memory_for(agent_id);

    let facts: Vec<Value> = if let Some(ref q) = query.q {
        layered
            .long_term
            .search(q, 20)
            .iter()
            .map(|fact| {
                serde_json::json!({
                    "content": fact.content,
                    "category": format!("{:?}", fact.category),
                    "source": fact.source,
                    "access_count": fact.access_count,
                })
            })
            .collect()
    } else if let Some(ref cat) = query.category {
        let category = match cat.as_str() {
            "preference" => crate::core::layered_memory::FactCategory::UserPreference,
            "habit" => crate::core::layered_memory::FactCategory::UserHabit,
            "tool" => crate::core::layered_memory::FactCategory::ToolResult,
            "decision" => crate::core::layered_memory::FactCategory::Decision,
            _ => crate::core::layered_memory::FactCategory::General,
        };
        layered
            .long_term
            .search_by_category(category, 20)
            .iter()
            .map(|fact| {
                serde_json::json!({
                    "content": fact.content,
                    "category": format!("{:?}", fact.category),
                    "source": fact.source,
                    "access_count": fact.access_count,
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    drop(core);
    ApiResponse::ok(serde_json::json!({
        "facts": facts,
        "total": facts.len(),
    }))
}

#[derive(Deserialize)]
pub struct MemorySearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub agent_id: Option<String>,
}

// ── Evals ──

pub async fn run_evals(
    State(state): State<AppState>,
) -> Json<ApiResponse<Value>> {
    let core = state.core.read().await;
    let suite = crate::core::evals::builtin_eval_suite();
    let messages = if let Some(sid) = core.session_mgr.current_id() {
        core.session_mgr.load_app_messages(&sid, 100)
    } else {
        Vec::new()
    };
    let tool_results: Vec<(String, String)> = messages
        .iter()
        .filter_map(|m| match m {
            crate::app::Message::ToolCall { name, result, .. } => Some((name.clone(), result.clone())),
            _ => None,
        })
        .collect();
    drop(core);

    let eval_results = suite.evaluate(&tool_results);
    let passed = eval_results.iter().filter(|r| r.passed).count();
    let total = eval_results.len();
    let avg_score = if total > 0 {
        eval_results.iter().map(|r| r.score).sum::<f64>() / total as f64
    } else {
        0.0
    };

    ApiResponse::ok(serde_json::json!({
        "suite": suite.name,
        "passed": passed,
        "total": total,
        "avg_score": avg_score,
        "results": eval_results,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建一个测试 AppState，必须在新线程中调用以避免嵌套 Runtime 问题。
    fn new_test_state() -> AppState {
        let (_cfg, core) = crate::test_helpers::test_core();
        AppState::new(core, "test-token".to_string())
    }

    /// 在独立线程中运行一个需要 AppState 的测试。
    fn run_state_test<F>(name: &str, f: F)
    where
        F: FnOnce(tokio::runtime::Runtime, AppState) + Send + 'static,
    {
        let result = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("创建测试运行时失败");
                let state = new_test_state();
                f(rt, state);
            })
            .expect("生成测试线程失败")
            .join();
        if let Err(e) = result {
            if let Some(msg) = e.downcast_ref::<&str>() {
                panic!("测试 '{}' panic: {}", name, msg);
            } else if let Some(msg) = e.downcast_ref::<String>() {
                panic!("测试 '{}' panic: {}", name, msg);
            } else {
                panic!("测试 '{}' panic (unknown)", name);
            }
        }
    }

    // ── 无状态测试 ──

    #[tokio::test]
    async fn test_health_handler() {
        let result = health().await;
        assert!(result.success, "health 应返回 success");
        assert_eq!(result.data, Some("OK"));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_api_response_ok() {
        let resp = ApiResponse::ok(serde_json::json!(["a", "b"]));
        assert!(resp.success);
        assert_eq!(resp.data, Some(serde_json::json!(["a", "b"])));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_api_response_err() {
        let resp: Json<ApiResponse<()>> = ApiResponse::err("something went wrong");
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("something went wrong".to_string()));
    }

    // ── 状态相关测试（在独立线程中运行） ──

    #[test]
    fn test_get_config_returns_sanitized() {
        run_state_test("test_get_config_returns_sanitized", |rt, state| {
            let result = rt.block_on(get_config(State(state)));
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert_eq!(data["provider"], "openai");
            assert_eq!(data["model"], "test-model");
            // 不应泄露 api_key
            assert!(data.get("api_key").is_none());
        });
    }

    #[test]
    fn test_get_current_session_no_session() {
        run_state_test("test_get_current_session_no_session", |rt, state| {
            let result = rt.block_on(get_current_session(State(state)));
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert!(data["id"].is_null(), "无会话时应返回 null id");
        });
    }

    #[test]
    fn test_create_session_returns_id() {
        run_state_test("test_create_session_returns_id", |rt, state| {
            let result = rt.block_on(create_session(
                State(state),
                Some(Json(serde_json::json!({"agent_id": "default"}))),
            ));
            assert!(result.success, "create_session 应成功");
            let data = result.0.data.unwrap();
            assert!(
                data["id"].as_str().map(|s| !s.is_empty()).unwrap_or(false),
                "应返回非空 id"
            );
            assert_eq!(data["agent_id"], "default");
        });
    }

    #[test]
    fn test_create_session_default_agent() {
        run_state_test("test_create_session_default_agent", |rt, state| {
            // 不传 agent_id，应默认为 "default"
            let result = rt.block_on(create_session(State(state), None));
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert_eq!(data["agent_id"], "default");
        });
    }

    #[test]
    fn test_list_sessions_after_create() {
        run_state_test("test_list_sessions_after_create", |rt, state| {
            // 先创建一个会话
            let _created = rt.block_on(create_session(
                State(state.clone()),
                Some(Json(serde_json::json!({"agent_id": "default"}))),
            ));
            // 然后列出会话
            let result = rt.block_on(list_sessions(State(state)));
            assert!(result.success);
            let sessions = result.0.data.unwrap();
            assert!(!sessions.is_empty(), "创建会话后列表不应为空");
            assert_eq!(sessions[0]["agent_id"], "default");
        });
    }

    #[test]
    fn test_send_message_missing_body() {
        run_state_test("test_send_message_missing_body", |rt, state| {
            let result = rt.block_on(send_message(State(state), Json(serde_json::json!({}))));
            assert!(!result.success, "缺少 message 时应返回错误");
            assert_eq!(result.error, Some("Missing 'message' field".to_string()));
        });
    }

    #[test]
    fn test_send_message_valid() {
        run_state_test("test_send_message_valid", |rt, state| {
            let result = rt.block_on(send_message(
                State(state),
                Json(serde_json::json!({"message": "hello"})),
            ));
            assert!(result.success, "有效消息应返回 success");
            let data = result.0.data.unwrap();
            assert_eq!(data["status"], "processing");
            assert!(
                !data["session_id"].as_str().unwrap_or("").is_empty(),
                "应返回非空 session_id"
            );
        });
    }

    #[test]
    fn test_send_message_with_agent_id() {
        run_state_test("test_send_message_with_agent_id", |rt, state| {
            let result = rt.block_on(send_message(
                State(state),
                Json(serde_json::json!({"message": "hi", "agent_id": "default"})),
            ));
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert_eq!(data["status"], "processing");
        });
    }

    #[test]
    fn test_list_tools_returns_schemas() {
        run_state_test("test_list_tools_returns_schemas", |rt, state| {
            let result = rt.block_on(list_tools(State(state)));
            assert!(result.success);
            let tools = result.0.data.unwrap();
            assert!(!tools.is_empty(), "应返回至少一个工具");
            // 每个 tool schema 应有 type/function 字段
            let first = &tools[0];
            assert_eq!(first["type"], "function");
            assert!(
                first["function"]["name"]
                    .as_str()
                    .map(|s| !s.is_empty())
                    .unwrap_or(false)
            );
        });
    }

    #[test]
    fn test_get_agents_returns_default() {
        run_state_test("test_get_agents_returns_default", |rt, state| {
            let result = rt.block_on(get_agents(State(state)));
            assert!(result.success);
            let agents = result.0.data.unwrap();
            assert!(!agents.is_empty(), "应返回至少 default agent");
            let default = agents.iter().find(|a| a["id"] == "default");
            assert!(default.is_some(), "应包含 default agent");
        });
    }

    #[test]
    fn test_get_agent_detail_default() {
        run_state_test("test_get_agent_detail_default", |rt, state| {
            let result = rt.block_on(get_agent_detail(State(state), Path("default".to_string())));
            assert!(result.success);
            let detail = result.0.data.unwrap();
            assert_eq!(detail["id"], "default");
            assert_eq!(detail["provider"], "openai");
        });
    }

    #[test]
    fn test_list_skills_returns_list() {
        run_state_test("test_list_skills_returns_list", |rt, state| {
            let result = rt.block_on(list_skills(State(state)));
            assert!(result.success);
            // 新安装环境下技能列表可能为空，但至少不应 panic
            let _skills = result.0.data.unwrap();
        });
    }
}
