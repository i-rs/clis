use crate::server::AppState;
use crate::server::UserId;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::Value;

/// Convert an app::Message to an API-compatible JSON value.
fn message_to_api_json(msg: &crate::app::Message) -> Value {
    match msg {
        crate::app::Message::User { text } => {
            serde_json::json!({"role": "user", "content": text})
        }
        crate::app::Message::Assistant {
            text, reasoning, ..
        } => {
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
        crate::app::Message::Image {
            path,
            alt_text,
            width,
            height,
            format,
        } => {
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
        crate::app::Message::Evaluation {
            tool,
            valid,
            issues,
        } => {
            serde_json::json!({
                "role": "evaluation",
                "content": format!("{}: {}", tool, if *valid { "✓" } else { "✗" }),
                "tool": tool,
                "valid": valid,
                "issues": issues,
            })
        }
        crate::app::Message::Quality {
            score,
            complete,
            references_valid,
            issues,
        } => {
            serde_json::json!({
                "role": "quality",
                "content": format!("质量评分: {}", score.unwrap_or(0.0)),
                "score": score.map(|s| s.to_string()),
                "complete": complete,
                "references_valid": references_valid,
                "issues": issues,
            })
        }
        crate::app::Message::Feedback { positive, message } => {
            serde_json::json!({
                "role": "feedback",
                "content": format!("positive: {}", positive),
                "positive": positive,
                "message": message,
            })
        }
        _ => serde_json::json!({"role": "unknown"}),
    }
}

/// Get current session info for the authenticated user.
pub async fn get_current_session(
    State(state): State<AppState>,
    UserId(user_id): UserId,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    // Resolve the user's session: check if global current_id belongs to them,
    // otherwise find their most recent session.
    let id = core
        .session_mgr
        .current_id()
        .filter(|sid| {
            core.session_mgr
                .session_meta(sid)
                .map(|m| m.user_id == user_id)
                .unwrap_or(false)
        })
        .map(|s| s.to_string())
        .or_else(|| {
            core.session_mgr
                .sessions()
                .iter()
                .rev()
                .find(|s| s.user_id == user_id)
                .map(|s| s.id.clone())
        });
    match id {
        Some(ref sid) => {
            let meta = core.session_mgr.session_meta(sid);
            let messages = core.session_mgr.load_app_messages_async(sid, 50).await;
            let msgs: Vec<Value> = messages.iter().map(message_to_api_json).collect();
            super::ApiResponse::ok(serde_json::json!({
                "id": sid,
                "title": meta.as_ref().map(|m| &m.title),
                "message_count": meta.as_ref().map(|m| m.message_count).unwrap_or(0),
                "messages": msgs,
                "agent_id": meta.as_ref().map(|m| &m.agent_id),
            }))
        }
        None => super::ApiResponse::ok(serde_json::json!({
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
    UserId(user_id): UserId,
    body: Option<Json<Value>>,
) -> Json<super::ApiResponse<Value>> {
    let agent_id = body
        .as_ref()
        .and_then(|b| b.get("agent_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    let mut core = state.core.write().await;
    let id = core
        .session_mgr
        .create_session_for_async(agent_id, &user_id)
        .await;
    super::ApiResponse::ok(serde_json::json!({
        "id": id,
        "title": "",
        "message_count": 0,
        "agent_id": agent_id,
    }))
}

/// Switch to an existing session.
pub async fn switch_session(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    let mut core = state.core.write().await;
    // Verify session belongs to this user
    if let Some(meta) = core.session_mgr.session_meta(&id)
        && meta.user_id != user_id
    {
        return super::ApiResponse::err("Session does not belong to you");
    }
    if core.session_mgr.switch_to(&id) {
        let meta = core.session_mgr.session_meta(&id);
        super::ApiResponse::ok(serde_json::json!({
            "id": id,
            "title": meta.as_ref().map(|m| &m.title),
            "message_count": meta.as_ref().map(|m| m.message_count).unwrap_or(0),
            "agent_id": meta.as_ref().map(|m| &m.agent_id),
        }))
    } else {
        super::ApiResponse::err("Session not found")
    }
}

/// List all sessions belonging to the authenticated user.
pub async fn list_sessions(
    State(state): State<AppState>,
    UserId(user_id): UserId,
) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    let sessions: Vec<Value> = core
        .session_mgr
        .sessions()
        .iter()
        .filter(|s| s.user_id == user_id)
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
    super::ApiResponse::ok(sessions)
}

/// Get session messages.
pub async fn get_session(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;

    // Verify session belongs to this user
    if let Some(meta) = core.session_mgr.session_meta(&id) {
        if meta.user_id != user_id {
            return super::ApiResponse::err("Session does not belong to you");
        }
    } else {
        return super::ApiResponse::err("Session not found");
    }

    let messages = core.session_mgr.load_app_messages_async(&id, 100).await;
    let msgs: Vec<Value> = messages.iter().map(message_to_api_json).collect();

    let meta = core.session_mgr.session_meta(&id);

    super::ApiResponse::ok(serde_json::json!({
        "id": id,
        "messages": msgs,
        "title": meta.as_ref().map(|m| &m.title),
        "agent_id": meta.as_ref().map(|m| &m.agent_id),
    }))
}

/// Delete a session.
pub async fn delete_session(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<String>,
) -> Json<super::ApiResponse<&'static str>> {
    let mut core = state.core.write().await;

    // Verify session belongs to this user
    let agent_id = match core.session_mgr.session_meta(&id) {
        Some(meta) if meta.user_id == user_id => meta.agent_id.clone(),
        Some(_) => return super::ApiResponse::err("Session does not belong to you"),
        None => return super::ApiResponse::err("Session not found"),
    };

    {
        if let Ok(layered) = core.agent_store.layered_memory_for_mut(&user_id, &agent_id) {
            layered.end_session();
        }
    }
    core.session_mgr.delete_session_async(&id).await;
    drop(core);
    super::ApiResponse::ok("deleted")
}

/// POST /api/sessions/{id}/feedback — record user feedback (thumbs up/down).
pub async fn post_session_feedback(
    State(state): State<AppState>,
    UserId(user_id): UserId,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Json<super::ApiResponse<&'static str>> {
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
    if let Ok(mem) = core.agent_store.memory_for_mut(&user_id, &agent_id) {
        mem.record_session_feedback(&id, positive);
    }

    // Append feedback to session via SessionManager.
    let msg = crate::app::Message::Feedback {
        positive,
        message: feedback_msg.map(|s| s.to_string()),
    };
    if let Err(e) = core.session_mgr.persist_messages_async(&id, &[msg]).await {
        tracing::error!("feedback persist failed: {}", e);
    }

    if let Ok(mem) = core.agent_store.memory_for_mut(&user_id, &agent_id) {
        mem.flush_async().await;
    }
    drop(core);

    super::ApiResponse::ok("ok")
}
