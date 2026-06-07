use crate::server::AppState;
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct MemoryQuery {
    pub agent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct MemorySearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub agent_id: Option<String>,
}

pub async fn get_layered_memory(
    State(state): State<AppState>,
    Query(query): Query<MemoryQuery>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let agent_id = query.agent_id.as_deref().unwrap_or("default");
    let layered = core.agent_store.layered_memory_for("default", agent_id);
    let summary = layered.format_for_prompt();
    let fact_count = layered.long_term.facts.len();
    let entity_count = layered.working.entities.len();
    drop(core);
    super::ApiResponse::ok(serde_json::json!({
        "agent_id": agent_id,
        "summary": summary,
        "document_count": fact_count,
        "entity_count": entity_count,
    }))
}

pub async fn clear_layered_memory(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<&'static str>> {
    let mut core = state.core.write().await;
    let agent_id = body
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let layered = core.agent_store.layered_memory_for_mut("default", agent_id);
    layered.working.clear();
    layered.long_term.facts.clear();
    layered.summaries.clear();
    drop(core);
    super::ApiResponse::ok("cleared")
}

pub async fn search_layered_memory(
    State(state): State<AppState>,
    Query(query): Query<MemorySearchQuery>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let agent_id = query.agent_id.as_deref().unwrap_or("default");
    let layered = core.agent_store.layered_memory_for("default", agent_id);

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
            "preference" => i_rs_claw_core::core::layered_memory::FactCategory::UserPreference,
            "habit" => i_rs_claw_core::core::layered_memory::FactCategory::UserHabit,
            "tool" => i_rs_claw_core::core::layered_memory::FactCategory::ToolResult,
            "decision" => i_rs_claw_core::core::layered_memory::FactCategory::Decision,
            _ => i_rs_claw_core::core::layered_memory::FactCategory::General,
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
    super::ApiResponse::ok(serde_json::json!({
        "facts": facts,
        "total": facts.len(),
    }))
}
