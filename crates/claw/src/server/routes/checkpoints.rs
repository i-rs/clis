use crate::server::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct CheckpointQuery {
    pub session_id: Option<String>,
}

pub async fn list_checkpoints(
    State(state): State<AppState>,
    Query(query): Query<CheckpointQuery>,
) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    let checkpoints: Vec<Value> = if let Ok(store) = core.checkpoint_store.lock() {
        store
            .list()
            .iter()
            .filter(|(id, _, _)| {
                query
                    .session_id
                    .as_ref()
                    .is_none_or(|sid| id.starts_with(&format!("cp_{}_", sid)))
            })
            .map(|(id, round, ts)| {
                serde_json::json!({
                    "id": id,
                    "round": round,
                    "timestamp": ts,
                })
            })
            .collect()
    } else {
        Vec::new()
    };
    drop(core);
    super::ApiResponse::ok(checkpoints)
}

pub async fn get_checkpoint_detail(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let result = {
        let Ok(store) = core.checkpoint_store.lock() else {
            drop(core);
            return super::ApiResponse::err("检查点存储不可用");
        };
        let Some(cp) = store.get(&id) else {
            drop(store);
            drop(core);
            return super::ApiResponse::err("检查点未找到");
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
    super::ApiResponse::ok(result)
}

pub async fn restore_checkpoint(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let session_id = body
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let round = body.get("round").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if session_id.is_empty() {
        return super::ApiResponse::err("需要 session_id 参数");
    }
    let cp_id = format!("cp_{}_{}", session_id, round);
    let core = state.core.read().await;
    let result = {
        let Ok(store) = core.checkpoint_store.lock() else {
            drop(core);
            return super::ApiResponse::err("检查点存储不可用");
        };
        let Some(cp) = store.get(&cp_id) else {
            drop(store);
            drop(core);
            return super::ApiResponse::err(&format!(
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
    super::ApiResponse::ok(result)
}
