pub mod agents;
pub mod chat;
pub mod checkpoints;
pub mod config;
pub mod evals;
pub mod guardrails;
pub mod health;
pub mod images;
pub mod memory;
pub mod sessions;
pub mod stats;
pub mod tools;

pub use agents::*;
pub use chat::*;
pub use checkpoints::*;
pub use config::*;
pub use evals::*;
pub use guardrails::*;
pub use health::*;
pub use images::*;
pub use memory::*;
pub use sessions::*;
pub use stats::*;
pub use tools::*;

use axum::Json;
use serde::Serialize;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use axum::extract::{Path, State};

    /// Creates a test AppState.
    fn new_test_state() -> AppState {
        let (_cfg, core) = crate::test_helpers::test_core();
        AppState::new(core, "test-token".to_string())
    }

    fn run_state_test<F, Fut>(name: &str, f: F)
    where
        F: FnOnce(AppState) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let result = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let state = new_test_state();
                i_rs_claw_core::utils::sync_block_on(f(state));
            })
            .expect("Failed to spawn test thread")
            .join();
        if let Err(e) = result {
            if let Some(msg) = e.downcast_ref::<&str>() {
                panic!("Test '{}' panicked: {}", name, msg);
            } else if let Some(msg) = e.downcast_ref::<String>() {
                panic!("Test '{}' panicked: {}", name, msg);
            } else {
                panic!("Test '{}' panicked (unknown)", name);
            }
        }
    }

    // ── Stateless tests ──

    #[tokio::test]
    async fn test_health_handler() {
        let result = health().await;
        assert!(result.success, "health should return success");
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

    // ── Stateful tests (run on dedicated thread via run_state_test) ──

    #[test]
    fn test_get_config_returns_sanitized() {
        run_state_test("test_get_config_returns_sanitized", |state| async move {
            let result = get_config(State(state)).await;
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert_eq!(data["provider"], "openai");
            assert_eq!(data["model"], "test-model");
            assert!(data.get("api_key").is_none());
        });
    }

    #[test]
    fn test_get_current_session_no_session() {
        run_state_test("test_get_current_session_no_session", |state| async move {
            let result = get_current_session(State(state)).await;
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert!(data["id"].is_null(), "no session should return null id");
        });
    }

    #[test]
    fn test_create_session_returns_id() {
        run_state_test("test_create_session_returns_id", |state| async move {
            let result = create_session(
                State(state),
                Some(Json(serde_json::json!({"agent_id": "default"}))),
            ).await;
            assert!(result.success, "create_session should succeed");
            let data = result.0.data.unwrap();
            assert!(
                data["id"].as_str().map(|s| !s.is_empty()).unwrap_or(false),
                "should return non-empty id"
            );
            assert_eq!(data["agent_id"], "default");
        });
    }

    #[test]
    fn test_create_session_default_agent() {
        run_state_test("test_create_session_default_agent", |state| async move {
            let result = create_session(State(state), None).await;
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert_eq!(data["agent_id"], "default");
        });
    }

    #[test]
    fn test_list_sessions_after_create() {
        run_state_test("test_list_sessions_after_create", |state| async move {
            let _created = create_session(
                State(state.clone()),
                Some(Json(serde_json::json!({"agent_id": "default"}))),
            ).await;
            let result = list_sessions(State(state)).await;
            assert!(result.success);
            let sessions = result.0.data.unwrap();
            assert!(!sessions.is_empty(), "list should not be empty after create");
            assert_eq!(sessions[0]["agent_id"], "default");
        });
    }

    #[test]
    fn test_send_message_missing_body() {
        run_state_test("test_send_message_missing_body", |state| async move {
            let result = send_message(State(state), Json(serde_json::json!({}))).await;
            assert!(!result.success, "missing message should return error");
            assert_eq!(result.error, Some("Missing 'message' field".to_string()));
        });
    }

    #[test]
    fn test_send_message_valid() {
        run_state_test("test_send_message_valid", |state| async move {
            let result = send_message(
                State(state),
                Json(serde_json::json!({"message": "hello"})),
            ).await;
            assert!(result.success, "valid message should return success");
            let data = result.0.data.unwrap();
            assert_eq!(data["status"], "processing");
            assert!(
                !data["session_id"].as_str().unwrap_or("").is_empty(),
                "should return non-empty session_id"
            );
        });
    }

    #[test]
    fn test_send_message_with_agent_id() {
        run_state_test("test_send_message_with_agent_id", |state| async move {
            let result = send_message(
                State(state),
                Json(serde_json::json!({"message": "hi", "agent_id": "default"})),
            ).await;
            assert!(result.success);
            let data = result.0.data.unwrap();
            assert_eq!(data["status"], "processing");
        });
    }

    #[test]
    fn test_list_tools_returns_schemas() {
        run_state_test("test_list_tools_returns_schemas", |state| async move {
            let result = list_tools(State(state)).await;
            assert!(result.success);
            let tools = result.0.data.unwrap();
            assert!(!tools.is_empty(), "should return at least one tool");
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
        run_state_test("test_get_agents_returns_default", |state| async move {
            let result = get_agents(State(state)).await;
            assert!(result.success);
            let agents = result.0.data.unwrap();
            assert!(!agents.is_empty(), "should return at least default agent");
            let default = agents.iter().find(|a| a["id"] == "default");
            assert!(default.is_some(), "should contain default agent");
        });
    }

    #[test]
    fn test_get_agent_detail_default() {
        run_state_test("test_get_agent_detail_default", |state| async move {
            let result = get_agent_detail(State(state), Path("default".to_string())).await;
            assert!(result.success);
            let detail = result.0.data.unwrap();
            assert_eq!(detail["id"], "default");
            assert_eq!(detail["provider"], "openai");
        });
    }

    #[test]
    fn test_list_skills_returns_list() {
        run_state_test("test_list_skills_returns_list", |state| async move {
            let result = list_skills(State(state)).await;
            assert!(result.success);
            let _skills = result.0.data.unwrap();
        });
    }
}
