pub mod agents;
pub mod chat;
pub mod checkpoints;
pub mod config;
pub mod evals;
pub mod guardrails;
pub mod health;
pub mod images;
pub mod mcp_config;
pub mod memory;
pub mod providers;
pub mod sessions;
pub mod settings;
pub mod stats;
pub mod tools;
pub mod users;

pub use agents::*;
pub use chat::*;
pub use checkpoints::*;
pub use config::*;
pub use evals::*;
pub use guardrails::*;
pub use health::*;
pub use images::*;
pub use mcp_config::*;
pub use memory::*;
pub use providers::*;
pub use sessions::*;
pub use settings::*;
pub use stats::*;
pub use tools::*;
pub use users::*;

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

/// Return an error response with a specific HTTP status code.
#[allow(dead_code)]
pub fn err_status<T: Serialize>(
    status: axum::http::StatusCode,
    msg: &str,
) -> (axum::http::StatusCode, Json<ApiResponse<T>>) {
    (status, ApiResponse::err(msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::AppState;
    use axum::extract::{Path, State};

use crate::server::UserId;

    /// Creates a test AppState.
    fn new_test_state() -> AppState {
        let (_cfg, core) = crate::test_helpers::test_core();
        AppState::new(core, "test-token".to_string())
    }

    fn test_uid() -> UserId {
        UserId("default".to_string())
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
            let result = get_current_session(State(state), test_uid()).await;
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
                test_uid(),
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
            let result = create_session(State(state), test_uid(), None).await;
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
                test_uid(),
                Some(Json(serde_json::json!({"agent_id": "default"}))),
            ).await;
            let result = list_sessions(State(state), test_uid()).await;
            assert!(result.success);
            let sessions = result.0.data.unwrap();
            assert!(!sessions.is_empty(), "list should not be empty after create");
            assert_eq!(sessions[0]["agent_id"], "default");
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
            let result = list_skills(State(state), test_uid()).await;
            assert!(result.success);
            let _skills = result.0.data.unwrap();
        });
    }

    #[test]
    fn test_update_config_invalid_provider_returns_400() {
        run_state_test("test_update_config_invalid_provider_returns_400", |state| async move {
            let (status, Json(body)) = update_config(
                State(state),
                Json(serde_json::json!({"provider": "not-a-real-provider"})),
            ).await;
            assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
            assert!(!body.success, "should report failure");
            assert!(
                body.error.as_ref().map(|e| e.contains("provider")).unwrap_or(false),
                "error should mention 'provider'; got {:?}",
                body.error
            );
        });
    }

    #[test]
    fn test_update_config_valid_provider_returns_200() {
        run_state_test("test_update_config_valid_provider_returns_200", |state| async move {
            let (status, Json(body)) = update_config(
                State(state),
                Json(serde_json::json!({"provider": "anthropic"})),
            ).await;
            assert_eq!(status, axum::http::StatusCode::OK);
            assert!(body.success, "should report success");
        });
    }
}
