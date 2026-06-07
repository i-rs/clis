use crate::server::AppState;
use crate::server::UserId;
use i_rs_claw_core::llm::LlmEvent;
#[cfg(feature = "dashboard")]
use i_rs_claw_core::message::MessageAccumulator;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
    response::sse::{Event, Sse},
};
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;
use tokio::sync::mpsc;

/// Build an SSE stream from an LlmEvent receiver with auto-incrementing event IDs.
fn build_sse_stream(
    rx: mpsc::UnboundedReceiver<LlmEvent>,
    state: AppState,
    user_id: String,
    sid: String,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = futures_util::stream::unfold(
        (Some(rx), state, user_id, sid, MessageAccumulator::new(), 0u64),
        |(rx_opt, state, user_id, sid, mut acc, mut seq)| async move {
            let mut rx = rx_opt?;
            loop {
                let event = rx.recv().await?;
                let sse_event;
                let mut keep_rx = true;

                match event {
                    LlmEvent::ToolExecuted { name, args, result, step, total_steps, category } => {
                        let mut core = state.core.write().await;
                        let i_rs_index = core.config.i_rs_tool_index.clone();
                        let agent_id = core.session_mgr.session_meta(&sid)
                            .map(|m| m.agent_id.clone()).unwrap_or_else(|| "default".to_string());
                        i_rs_claw_core::core::record_tool_memory(
                            &user_id, &mut core.agent_store, &i_rs_index, &agent_id, &name, &args, &result,
                        );
                        if !category.is_retryable_or_fatal() {
                            i_rs_claw_core::core::record_layered_tool_memory(
                                &user_id, &mut core.agent_store, &agent_id, &name, &result,
                            );
                        }
                        drop(core);
                        let data = serde_json::to_string(&serde_json::json!({
                            "name": name, "args": args, "result": result,
                            "step": step, "total_steps": total_steps,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("tool_executed").data(data).id(seq.to_string());
                        acc.apply(&LlmEvent::ToolExecuted { name: name.clone(), args: args.clone(), result: result.clone(), step, total_steps, category });
                    }
                    LlmEvent::Done(msgs, usage, _trace_id) => {
                        let mut core = state.core.write().await;
                        let agent_id = core.session_mgr.session_meta(&sid)
                            .map(|m| m.agent_id.clone()).unwrap_or_else(|| "default".to_string());
                        acc.apply(&LlmEvent::Done(msgs.clone(), usage, String::new()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        if let Err(e) = core.session_mgr.persist_messages(&sid, &finalized) {
                            tracing::error!("persist_messages (Done) 失败: {}", e);
                        }
                        core.session_mgr.save_api_messages(&sid, &msgs);
                        let quality_msg = core.evaluate_completed_session(&sid);
                        let quality_json = match &quality_msg {
                            Some(crate::app::Message::Quality { score, complete, issues, references_valid }) => {
                                serde_json::json!({"score": score.map(|s| s.to_string()).unwrap_or_default(), "complete": complete, "issues": issues, "references_valid": references_valid})
                            }
                            _ => serde_json::json!(null),
                        };
                        if let Some(q) = &quality_msg {
                            if let Err(e) = core.session_mgr.persist_messages(&sid, &[q.clone()]) {
                                tracing::error!("quality 持久化失败: {}", e);
                            }
                        }
                        core.agent_store.memory_for_mut(&user_id, &agent_id)
                            .expect("BUG: default agent runtime not initialized")
                            .flush();
                        drop(core);
                        state.chat_concurrency.release();
                        let done_json = serde_json::json!({"usage": usage, "quality": quality_json, "session_id": &sid});
                        let data = serde_json::to_string(&done_json).unwrap_or_default();
                        sse_event = Event::default().event("done").data(data).id(seq.to_string());
                        keep_rx = false;
                    }
                    LlmEvent::Error(e) => {
                        acc.apply(&LlmEvent::Error(e.clone()));
                        let finalized = acc.into_messages();
                        acc = MessageAccumulator::new();
                        {
                            let mut core = state.core.write().await;
                            core.session_mgr.mark_error(&sid, &e);
                            if let Err(err) = core.session_mgr.persist_messages(&sid, &finalized) {
                                tracing::error!("persist_messages (error) 失败: {}", err);
                            }
                        }
                        state.chat_concurrency.release();
                        sse_event = Event::default().event("error").data(e).id(seq.to_string());
                        keep_rx = false;
                    }
                    LlmEvent::Token(t) => {
                        acc.apply(&LlmEvent::Token(t.clone()));
                        sse_event = Event::default().event("token").data(t).id(seq.to_string());
                    }
                    LlmEvent::Reasoning(t) => {
                        acc.apply(&LlmEvent::Reasoning(t.clone()));
                        sse_event = Event::default().event("reasoning").data(t).id(seq.to_string());
                    }
                    LlmEvent::Status(s) => {
                        sse_event = Event::default().event("status").data(s).id(seq.to_string());
                    }
                    LlmEvent::NewRound(_) => {
                        acc.apply(&event);
                        sse_event = Event::default().event("new_round").data("").id(seq.to_string());
                    }
                    LlmEvent::ImageGenerated { path, alt_text, format, width, height } => {
                        acc.apply(&LlmEvent::ImageGenerated { path: path.clone(), alt_text: alt_text.clone(), format: format.clone(), width, height });
                        let data = serde_json::to_string(&serde_json::json!({
                            "path": path, "alt_text": alt_text, "format": format, "width": width, "height": height,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("image_generated").data(data).id(seq.to_string());
                    }
                    LlmEvent::Evaluation { tool, valid, issues } => {
                        acc.apply(&LlmEvent::Evaluation { tool: tool.clone(), valid, issues: issues.clone() });
                        let data = serde_json::to_string(&serde_json::json!({
                            "tool": tool, "valid": valid, "issues": issues,
                        })).unwrap_or_default();
                        sse_event = Event::default().event("evaluation").data(data).id(seq.to_string());
                    }
                    _ => continue,
                }
                seq += 1;
                let (next_rx, next_acc) = if keep_rx { (Some(rx), acc) } else { (None, MessageAccumulator::new()) };
                return Some((Ok::<_, Infallible>(sse_event), (next_rx, state, user_id, sid, next_acc, seq)));
            }
        },
    );
    Sse::new(stream)
}

/// Send a message and start LLM processing.
/// Returns the session ID so the client can subscribe to SSE events.
#[allow(dead_code)]
pub async fn send_message(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let text = match body.get("message").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return super::ApiResponse::err("Missing 'message' field"),
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
        core.session_mgr.create_session_for(&agent_id, &user_id);
    }

    let sid = match core.session_mgr.current_id() {
        Some(id) => id.to_string(),
        None => return super::ApiResponse::err("没有活跃会话"),
    };

    // Save user message via SessionManager (maintains cursor + message_count).
    {
        let msg = crate::app::Message::User { text: text.clone() };
        if let Err(e) = core.session_mgr.persist_messages(&sid, &[msg]) {
            tracing::error!("user message persist failed: {}", e);
        }
    }

    {
        let layered = match core.agent_store.layered_memory_for_mut(&user_id, &agent_id) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!(error = %e, "agent lookup failed");
                return super::ApiResponse::err(&format!("Agent not initialized: {}", e));
            }
        };
        layered.record_user_statement(&text);
    }

    // Drop the write lock — chat_stream will spawn the actual LLM call.
    drop(core);

    super::ApiResponse::ok(serde_json::json!({
        "session_id": sid,
        "status": "processing",
    }))
}

/// Single-endpoint chat: POST body → SSE stream directly.
/// Unifies send_message + chat_stream into one endpoint.
pub async fn chat(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Json(body): Json<Value>,
) -> axum::response::Response {
    if let Err(e) = state.chat_concurrency.try_acquire() {
        let err = serde_json::json!({"success": false, "error": e});
        return (axum::http::StatusCode::TOO_MANY_REQUESTS, axum::Json(err)).into_response();
    }

    let text = match body.get("message").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            state.chat_concurrency.release();
            let err = serde_json::json!({"success": false, "error": "Missing 'message'"});
            return (axum::http::StatusCode::BAD_REQUEST, axum::Json(err)).into_response();
        }
    };

    let agent_id = body
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let (llm_tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    let sid = {
        let mut core = state.core.write().await;

        let session_id = core
            .session_mgr
            .current_id()
            .map(|id| id.to_string())
            .unwrap_or_default();

        if session_id.is_empty() {
            core.session_mgr.create_session_for(&agent_id, &user_id);
        }

        let sid = core
            .session_mgr
            .current_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| {
                // Fallback: create a fresh session
                core.session_mgr.create_session_for(&agent_id, &user_id);
                core.session_mgr
                    .current_id()
                    .map(|id| id.to_string())
                    .unwrap_or_default()
            });

        // Persist user message
        let msg = crate::app::Message::User { text: text.clone() };
        if let Err(e) = core.session_mgr.persist_messages(&sid, &[msg]) {
            tracing::error!("user message persist failed: {}", e);
        }

        {
            let layered = match core.agent_store.layered_memory_for_mut(&user_id, &agent_id) {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!(error = %e, "agent lookup failed");
                    state.chat_concurrency.release();
                    let err = serde_json::json!({"success": false, "error": format!("Agent not initialized: {}", e)});
                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, axum::Json(err)).into_response();
                }
            };
            layered.record_user_statement(&text);
        }

        // Build messages and spawn chat_loop
        let records = core.session_mgr.load_app_messages(&sid, 50);
        let msgs = core.build_messages_from_log(&records, &agent_id);
        let recent: Vec<Value> = records
            .iter()
            .filter_map(|m| match m {
                crate::app::Message::User { text } => {
                    Some(serde_json::json!({"role":"user","content":text}))
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    Some(serde_json::json!({"role":"assistant","content":text}))
                }
                _ => None,
            })
            .collect();

        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
        drop(core);
        sid
    };

    build_sse_stream(rx, state.clone(), user_id, sid).into_response()
}

/// Legacy SSE stream endpoint (backward compatible).
pub async fn chat_stream(
    State(state): State<AppState>,
    UserId(user_id): UserId,
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

        let records = core.session_mgr.load_app_messages(&session_id, 50);
        let msgs = core.build_messages_from_log(&records, &agent_id);
        let recent: Vec<Value> = records
            .iter()
            .filter_map(|m| match m {
                crate::app::Message::User { text } => {
                    Some(serde_json::json!({"role":"user","content":text}))
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    Some(serde_json::json!({"role":"assistant","content":text}))
                }
                _ => None,
            })
            .collect();

        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
    }

    build_sse_stream(rx, state.clone(), user_id, session_id).into_response()
}

/// Resume an SSE stream after disconnection.
/// Accepts `?cursor=N` (last received event ID) via query params.
/// Returns a fresh stream. Client handles deduplication via event IDs.
pub async fn chat_stream_resume(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(session_id): Path<String>,
    Query(query): Query<ResumeQuery>,
) -> axum::response::Response {
    let _cursor = query.cursor.unwrap_or(0);

    let (llm_tx, rx) = mpsc::unbounded_channel::<LlmEvent>();

    {
        let mut core = state.core.write().await;
        core.session_mgr.switch_to(&session_id);

        let agent_id = core
            .session_mgr
            .session_meta(&session_id)
            .map(|m| m.agent_id.clone())
            .unwrap_or_else(|| "default".to_string());

        let records = core.session_mgr.load_app_messages(&session_id, 50);
        let msgs = core.build_messages_from_log(&records, &agent_id);
        let recent: Vec<Value> = records
            .iter()
            .filter_map(|m| match m {
                crate::app::Message::User { text } => {
                    Some(serde_json::json!({"role":"user","content":text}))
                }
                crate::app::Message::Assistant { text, .. } if !text.is_empty() => {
                    Some(serde_json::json!({"role":"assistant","content":text}))
                }
                _ => None,
            })
            .collect();

        core.spawn_chat_for_async(llm_tx, msgs, &agent_id, &recent);
    }

    build_sse_stream(rx, state.clone(), user_id, session_id).into_response()
}

#[derive(Deserialize)]
pub struct ResumeQuery {
    pub cursor: Option<u64>,
}
