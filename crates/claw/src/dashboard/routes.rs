use crate::dashboard::AppState;
use crate::llm::{LlmEvent, StreamResult};
use crate::mcp::McpRegistry;
use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::Value;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Lock the core mutex safely, returning early with an error on poison.
/// Use this in handler functions that return `Json<ApiResponse<...>>`.
macro_rules! lock_core {
    ($state:expr) => {
        $state.core.lock().await
    };
}

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
    let core = lock_core!(state);
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

    let mut core = lock_core!(state);

    // Create or get a session
    let session_id = core
        .session_mgr
        .current_id()
        .map(|id| id.to_string())
        .unwrap_or_default();

    if session_id.is_empty() {
        core.session_mgr.create_session_for(&agent_id);
    }

    let sid = match core
        .session_mgr
        .current_id() {
            Some(id) => id.to_string(),
            None => return ApiResponse::err("没有活跃会话"),
        };

    // Save user message
    core.session_mgr.append_message("user", &text, None);

    // Drop the lock before returning
    drop(core);

    ApiResponse::ok(serde_json::json!({
        "session_id": sid,
        "status": "processing",
    }))
}

/// Build API-compatible messages from JSONL session records, using the
/// system prompt and tool index for the given agent.
fn build_dashboard_messages(core: &crate::core::AppCore, session_id: &str, agent_id: &str) -> Vec<Value> {
    let records = core.session_mgr.load_messages(session_id, 50);
    let resolved = core.config.agent_config(agent_id);

    let system_prompt = resolved.system_prompt.unwrap_or_else(|| {
        let enabled = if resolved.enabled_tools.is_empty() { None } else { Some(&resolved.enabled_tools) };
        let memory = core.agent_store.memory_for(agent_id);
        let tool_cache = core.agent_store.tool_cache_for(agent_id);
        let skill_store = core.agent_store.skill_store_for(agent_id);
        crate::core::engine::builder::build_system_prompt(
            &crate::tools::format_index(enabled),
            &memory.format_hot_tools(tool_cache),
            &skill_store.format_skills(),
            &memory.format_user_memory(),
            &memory.format_user_profile(),
            core.config.execution_mode == crate::config::ExecutionMode::PlanThenExecute,
        )
    });

    let mut msgs = vec![serde_json::json!({ "role": "system", "content": system_prompt })];

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

// ── Dashboard chat loop (multi-round with tool execution) ──

/// Run the multi-round dashboard chat loop and save results.
#[allow(clippy::too_many_arguments)]
async fn dashboard_chat_loop(
    provider: Box<dyn crate::providers::LlmProvider>,
    mut msgs: Vec<Value>,
    tx: mpsc::UnboundedSender<LlmEvent>,
    state: AppState,
    session_id: String,
    enabled_tools: Option<std::collections::HashSet<String>>,
    mcp: McpRegistry,
    skills: Vec<crate::skill_store::SkillDefinition>,
) {
    use crate::utils::smart_truncate;

    // Build ToolContext for tool execution
    let tool_ctx = {
        let core = state.core.lock().await;
        crate::tools::ToolContext {
            config: core.config.clone(),
            mcp: mcp.clone(),
        }
    };

    // Create shared ToolCallExecutor
    let executor = crate::core::executor::ToolCallExecutor::new(tool_ctx, mcp.clone(), skills.clone());

    // Build tool schemas (same as chat_stream did before spawning)
    let tool_schemas = {
        let enabled = if enabled_tools.as_ref().is_none_or(|t| t.is_empty()) {
            None
        } else {
            enabled_tools.as_ref()
        };
        let mut schemas = crate::tools::ToolRegistry::with_skills(&skills).enabled_schemas(enabled);
        // Append MCP tool schemas if available
        for (client_idx, tool_def) in &mcp.tools {
            if let Some(_client) = mcp.clients.get(*client_idx) {
                let schema = crate::tools::mcp_tools::mcp_schema_to_openai(tool_def);
                schemas.push(schema);
            }
        }
        schemas
    };

    let mut round_count = 0u32;
    const MAX_ROUNDS: u32 = 20;

    loop {
        round_count += 1;
        if round_count > MAX_ROUNDS {
            let _ = tx.send(LlmEvent::Error(
                "已达最大执行轮数限制 (20)，已停止循环。".to_string(),
            ));
            break;
        }
        let _ = tx.send(LlmEvent::NewRound);
        let _ = tx.send(LlmEvent::Status("🤔 思考中…".to_string()));

        match provider.stream_chat(&msgs, &tool_schemas, &tx).await {
            Ok(StreamResult::Text(usage, text)) => {
                if !text.is_empty() {
                    msgs.push(serde_json::json!({
                        "role": "assistant",
                        "content": text,
                    }));
                }
                let _ = tx.send(LlmEvent::Done(Arc::new(msgs.clone()), usage));

                // Save to session
                let mut core = state.core.lock().await;
                core.session_mgr.append_message("assistant", &text, None);
                core.session_mgr.save_api_messages(&session_id, &msgs);
                break;
            }
            Ok(StreamResult::ToolCalls(calls, reasoning_content)) => {
                let tool_calls_array: Vec<Value> = calls
                    .iter()
                    .map(|(tc, _)| {
                        serde_json::json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments,
                            }
                        })
                    })
                    .collect();

                let mut assistant_msg = serde_json::json!({
                    "role": "assistant",
                    "content": null,
                    "tool_calls": tool_calls_array,
                });
                // DeepSeek requires reasoning_content to be echoed back
                if !reasoning_content.is_empty() {
                    assistant_msg["reasoning_content"] = serde_json::Value::String(reasoning_content);
                }
                msgs.push(assistant_msg);

                // Parallel execute all tool calls via ToolCallExecutor
                let total = calls.len();
                let _ = tx.send(LlmEvent::Status(format!("⚡ 执行 {} 个工具...", total)));

                let all_results = executor.execute(calls, &tx).await;

                // Push tool results to messages
                for result in &all_results {
                    msgs.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": result.call.id,
                        "content": smart_truncate(&result.result, 500),
                    }));
                }
                // Continue loop: send tool results back to LLM
            }
            Err(e) => {
                let _ = tx.send(LlmEvent::Error(format!("{}", e)));
                break;
            }
        }
    }

    // Attempt to sync app messages to JSONL after the full loop
    let core = state.core.lock().await;
    if let Some(api_msgs) = core.session_mgr.load_api_messages(&session_id) {
        // Convert API msgs to JSONL records, preserving tool call info
        let mut records: Vec<Value> = Vec::with_capacity(api_msgs.len());
        let mut i = 0;
        while i < api_msgs.len() {
            let m = &api_msgs[i];
            let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
            match role {
                "user" => {
                    records.push(serde_json::json!({
                        "type": "user",
                        "text": m.get("content").and_then(|c| c.as_str()).unwrap_or(""),
                    }));
                    i += 1;
                }
                "assistant" => {
                    let text = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    if m.get("tool_calls").and_then(|t| t.as_array()).is_some() {
                        // Tool call: pair with the next tool result
                        if let Some(tc_array) = m.get("tool_calls").and_then(|t| t.as_array()) {
                            for tc in tc_array {
                                let name = tc.get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("");
                                let args = tc.get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|a| a.as_str())
                                    .unwrap_or("");
                                // Look ahead for the tool result
                                let result = if i + 1 < api_msgs.len()
                                    && api_msgs[i + 1].get("role").and_then(|r| r.as_str()) == Some("tool")
                                {
                                    api_msgs[i + 1].get("content")
                                        .and_then(|c| c.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                } else {
                                    String::new()
                                };
                                records.push(serde_json::json!({
                                    "type": "tool_call",
                                    "name": name,
                                    "args": args,
                                    "result": result,
                                }));
                            }
                        }
                        // Skip both assistant(tool_calls) and tool result
                        i += 2;
                    } else if !text.is_empty() && text != "null" {
                        records.push(serde_json::json!({
                            "type": "assistant",
                            "text": text,
                        }));
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                _ => {
                    // Skip tool results (already handled via pairing above)
                    i += 1;
                }
            }
        }
        if !records.is_empty() {
            core.session_mgr.save_all_messages(&session_id, &records);
        }
    }
}

/// SSE stream for chat responses.
pub async fn chat_stream(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> axum::response::Response {
    let (msgs, provider, sid, enabled_tools, mcp, skills) = {
        let mut core = state.core.lock().await;
        core.session_mgr.switch_to(&session_id);

        // Read agent_id from session meta, defaulting to "default"
        let agent_id = core
            .session_mgr
            .session_meta(&session_id)
            .map(|m| m.agent_id.clone())
            .unwrap_or_else(|| "default".to_string());

        let msgs = build_dashboard_messages(&core, &session_id, &agent_id);
        let resolved = core.config.agent_config(&agent_id);
        let provider = crate::providers::create_provider_for(
            &resolved.provider,
            &resolved.api_key,
            &resolved.base_url,
            &resolved.model,
        );
        let enabled_tools = Some(resolved.enabled_tools.clone());
        let mcp = core.agent_store.mcp_registry_for(&agent_id).clone();
        let skills = core.agent_store.skill_store_for(&agent_id).executable_skills();

        (msgs, provider, session_id.clone(), enabled_tools, mcp, skills)
    };

    let (tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    let loop_state = state.clone();
    let loop_sid = sid.clone();

    tokio::spawn(async move {
        dashboard_chat_loop(provider, msgs, tx, loop_state, loop_sid, enabled_tools, mcp, skills).await;
    });

    let stream = futures_util::stream::unfold(Some(rx), |rx_opt| async move {
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
                LlmEvent::NewRound => {
                    let sse = Event::default().event("new_round").data("");
                    return Some((Ok::<_, Infallible>(sse), Some(rx)));
                }
                LlmEvent::ToolExecuted {
                    name,
                    args,
                    result,
                    step,
                    total_steps,
                } => {
                    let data = serde_json::to_string(&serde_json::json!({
                        "name": name,
                        "args": args,
                        "result": result,
                        "step": step,
                        "total_steps": total_steps,
                    }))
                    .unwrap_or_default();
                    let sse = Event::default().event("tool_executed").data(data);
                    return Some((Ok::<_, Infallible>(sse), Some(rx)));
                }
                _ => continue,
        }
    }
    });

    Sse::new(stream).into_response()
}

/// Get current session info.
pub async fn get_current_session(
    State(state): State<AppState>,
) -> Json<ApiResponse<Value>> {
    let core = lock_core!(state);
    let id = core.session_mgr.current_id().map(|s| s.to_string());
    match id {
        Some(ref sid) => {
            let meta = core.session_mgr.session_meta(sid);
            let messages = core.session_mgr.load_app_messages(sid, 50);
            let msgs: Vec<Value> = messages
                .iter()
                .map(|m| match m {
                    crate::app::Message::User { text } => {
                        serde_json::json!({"role": "user", "content": text})
                    }
                    crate::app::Message::Assistant { text } => {
                        serde_json::json!({"role": "assistant", "content": text})
                    }
                    crate::app::Message::ToolCall { name, args, result, .. } => {
                        serde_json::json!({"role": "tool_call", "name": name, "args": args, "result": result})
                    }
                    _ => serde_json::json!({"role": "unknown"}),
                })
                .collect();
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

    let mut core = lock_core!(state);
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
    let mut core = lock_core!(state);
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
pub async fn list_sessions(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Value>>> {
    let core = lock_core!(state);
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
    let core = lock_core!(state);
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
            crate::app::Message::ToolCall { name, args, result, .. } => {
                serde_json::json!({"role": "tool_call", "name": name, "args": args, "result": result})
            }
            _ => serde_json::json!({"role": "unknown"}),
        })
        .collect();

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
    let mut core = lock_core!(state);
    core.session_mgr.delete_session(&id);
    drop(core);
    ApiResponse::ok("deleted")
}

/// List available agent profiles.
pub async fn get_agents(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Value>>> {
    let core = lock_core!(state);
    let agent_ids = core.config.all_agent_ids();
    let agents: Vec<Value> = agent_ids
        .iter()
        .map(|id| {
            let resolved = core.config.agent_config(id);
            let tools: Vec<&String> = resolved.enabled_tools.iter().collect();
            let is_sub = id != "default" && !core.config.agents.contains_key(id);
            serde_json::json!({
                "id": id,
                "provider": resolved.provider,
                "model": resolved.model,
                "base_url": resolved.base_url,
                "tool_count": resolved.enabled_tools.len(),
                "enabled_tools": tools,
                "system_prompt": resolved.system_prompt,
                "is_sub_agent": is_sub,
                "capabilities": resolved.capabilities,
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
    let core = lock_core!(state);
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

    let mut core = lock_core!(state);

    // Get existing agent config
    let existing = match core.config.agents.get(&id) {
        Some(a) => a.clone(),
        None => return ApiResponse::err(&format!("Agent '{}' not found", id)),
    };

    // Merge body with existing (only override provided fields)
    let agent_config = crate::config::AgentConfig {
        provider: body.get("provider").and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.provider),
        api_key: body.get("api_key").and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.api_key),
        base_url: body.get("base_url").and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.base_url),
        model: body.get("model").and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.model),
        enabled_tools: body.get("enabled_tools")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .or(existing.enabled_tools),
        system_prompt: body.get("system_prompt").and_then(|v| v.as_str()).map(|s| s.to_string()).or(existing.system_prompt),
        system_prompt_file: existing.system_prompt_file,
        mcp_servers: None, // inherit from existing via merge
        allowed_dirs: None,
        capabilities: existing.capabilities,
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

    let mut core = lock_core!(state);

    // Check if agent already exists
    if core.config.agents.contains_key(&agent_id) {
        return ApiResponse::err(&format!("Agent '{}' already exists", agent_id));
    }

    // Build agent config from request body (all optional)
    let agent_config = crate::config::AgentConfig {
        provider: body.get("provider").and_then(|v| v.as_str()).map(|s| s.to_string()),
        api_key: body.get("api_key").and_then(|v| v.as_str()).map(|s| s.to_string()),
        base_url: body.get("base_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
        model: body.get("model").and_then(|v| v.as_str()).map(|s| s.to_string()),
        enabled_tools: body.get("enabled_tools")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        system_prompt: body.get("system_prompt").and_then(|v| v.as_str()).map(|s| s.to_string()),
        system_prompt_file: None,
        mcp_servers: None,
        allowed_dirs: None,
        capabilities: Vec::new(),
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
    core.agent_store.add_agent(&config, &claw_dir, &agent_id);

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

    let mut core = lock_core!(state);

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
pub async fn list_tools(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Value>>> {
    let core = lock_core!(state);
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

/// List user-defined skills with parsed metadata.
pub async fn list_skills(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<crate::skill_store::SkillDefinition>>> {
    let core = lock_core!(state);
    let store = core.agent_store.skill_store_for("default");
    let entries = store.list_skills();
    let skills: Vec<crate::skill_store::SkillDefinition> = entries
        .iter()
        .map(|e| crate::skill_store::SkillDefinition {
            name: e.name.clone(),
            description: crate::skill_store::parse_frontmatter(&e.content)
                .0
                .and_then(|t| t.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| e.name.clone()),
            parameters: crate::skill_store::parse_frontmatter(&e.content)
                .0
                .and_then(|t| t.get("parameters").and_then(|v| serde_json::to_value(v).ok())),
            content: crate::skill_store::parse_frontmatter(&e.content).1.to_string(),
        })
        .collect();
    ApiResponse::ok(skills)
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
                let rt = tokio::runtime::Runtime::new()
                    .expect("创建测试运行时失败");
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
            let result = rt.block_on(send_message(
                State(state),
                Json(serde_json::json!({})),
            ));
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
            assert!(!data["session_id"].as_str().unwrap_or("").is_empty(), "应返回非空 session_id");
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
            assert!(first["function"]["name"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
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
            let result = rt.block_on(get_agent_detail(
                State(state),
                Path("default".to_string()),
            ));
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
