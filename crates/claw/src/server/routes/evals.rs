use crate::server::AppState;
use axum::{
    Json,
    extract::State,
};
use serde_json::Value;

pub async fn run_evals(State(state): State<AppState>) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;
    let suite = i_rs_claw_core::core::evals::builtin_eval_suite();
    let messages = if let Some(sid) = core.session_mgr.current_id() {
        core.session_mgr.load_app_messages_async(sid, 100).await
    } else {
        Vec::new()
    };
    let tool_results: Vec<(String, String)> = messages
        .iter()
        .filter_map(|m| match m {
            crate::app::Message::ToolCall { name, result, .. } => {
                Some((name.clone(), result.clone()))
            }
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

    super::ApiResponse::ok(serde_json::json!({
        "suite": suite.name,
        "passed": passed,
        "total": total,
        "avg_score": avg_score,
        "results": eval_results,
    }))
}
