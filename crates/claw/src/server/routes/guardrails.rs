use axum::Json;
use serde_json::Value;

pub async fn check_guardrails(Json(body): Json<Value>) -> Json<super::ApiResponse<Value>> {
    let input = body.get("input").and_then(|v| v.as_str());
    let output = body.get("output").and_then(|v| v.as_str());
    let tool_name = body.get("tool_name").and_then(|v| v.as_str());
    let tool_args = body.get("tool_args");

    let mut results = Vec::new();

    let mgr = i_rs_claw_core::tools::guardrails::GuardrailManager::new();

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
        return super::ApiResponse::err("需要 input, output 或 tool_name+tool_args 参数");
    }

    let all_allowed = results
        .iter()
        .all(|r| r["allowed"].as_bool().unwrap_or(false));
    super::ApiResponse::ok(serde_json::json!({
        "allowed": all_allowed,
        "checks": results,
    }))
}
