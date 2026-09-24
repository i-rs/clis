use claw_core_storage_tests::conversations::runner::executor::ToolCallInfo;
use claw_core_storage_tests::conversations::runner::{
    ReplyCheck, Script, ScriptMeta, ScriptRunner, ScriptStep, StorageCheck,
    executor::{MockSession, StepOutput},
};
use i_rs_claw_core::app::Message;
use i_rs_claw_core::session::SessionManager;
use i_rs_claw_core::storage::ClawStorage;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper: create a temp File storage backend.
fn file_storage() -> (Arc<ClawStorage>, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(ClawStorage::file(dir.path().join("claw")));
    (storage, dir)
}

/// Helper: create a session in the storage and return its ID.
async fn create_session(storage: &Arc<ClawStorage>) -> String {
    let mut mgr = SessionManager::with_storage(storage.clone()).unwrap();
    mgr.create_session_for("default", "default")
}

#[tokio::test]
async fn test_storage_verification_session_created() {
    let (storage, _dir) = file_storage();
    let session_id = create_session(&storage).await;

    let script = Script {
        meta: ScriptMeta {
            tool: "i-rs-kv".into(),
            name: "verify_session_created".into(),
            description: "Verify session creation is reflected in storage".into(),
            required_capabilities: vec!["add".into()],
            storage_backends: vec!["file".into()],
            tags: vec!["storage-verify".into()],
        },
        steps: vec![ScriptStep {
            step: 1,
            title: "检查会话".into(),
            user_message: "存一下 blog_url".into(),
            expected_tool: Some("i-rs-kv".into()),
            expected_command: Some("add".into()),
            expected_args: Some(HashMap::from([("KEY".into(), "blog_url".into())])),
            expected_flags: None,
            check_reply: Some(ReplyCheck {
                contains: vec!["已记录".into()],
            }),
            verify_storage: Some(StorageCheck {
                file_check: None,
                expected_state: Some(serde_json::json!({
                    "session": { "exists": true, "length": 1 },
                })),
                no_new_records: None,
            }),
        }],
    };

    let mut session = MockSession::new(vec![StepOutput {
        reply: "已记录 blog_url".into(),
        tool_calls: vec![
            ToolCallInfo::new("i-rs-kv")
                .with_command("add")
                .with_arg("KEY", "blog_url")
                .with_arg("VALUE", "https://example.com"),
        ],
        prompt_tokens: 100,
        completion_tokens: 30,
        session_id: session_id.clone(),
    }]);

    let runner = ScriptRunner::new(script);
    let result = runner.run_with_storage(&mut session, Some(&storage)).await;

    assert!(result.passed, "session created verification should pass");
}

#[tokio::test]
async fn test_storage_verification_message_count() {
    let (storage, _dir) = file_storage();
    let session_id = create_session(&storage).await;

    // Add a message to the session to verify count
    {
        let msg = Message::User {
            text: "存一下 blog_url".into(),
        };
        storage
            .message_log
            .append_one(&session_id, &msg)
            .await
            .unwrap();
    }

    let script = Script {
        meta: ScriptMeta {
            tool: "i-rs-kv".into(),
            name: "verify_message_count".into(),
            description: "Verify message count in storage".into(),
            required_capabilities: vec!["list".into()],
            storage_backends: vec!["file".into()],
            tags: vec!["storage-verify".into()],
        },
        steps: vec![ScriptStep {
            step: 1,
            title: "列出记录".into(),
            user_message: "列出我的记录".into(),
            expected_tool: Some("i-rs-kv".into()),
            expected_command: Some("list".into()),
            expected_args: None,
            expected_flags: None,
            check_reply: None,
            verify_storage: Some(StorageCheck {
                file_check: None,
                expected_state: Some(serde_json::json!({
                    "session": { "exists": true },
                    "messages": { "min": 1 },
                })),
                no_new_records: None,
            }),
        }],
    };

    let mut session = MockSession::new(vec![StepOutput {
        reply: "已列出".into(),
        tool_calls: vec![ToolCallInfo::new("i-rs-kv").with_command("list")],
        prompt_tokens: 50,
        completion_tokens: 10,
        session_id: session_id.clone(),
    }]);

    let runner = ScriptRunner::new(script);
    let result = runner.run_with_storage(&mut session, Some(&storage)).await;

    assert!(result.passed, "message count verification should pass");
}

#[tokio::test]
async fn test_storage_verification_tool_cache() {
    let (storage, _dir) = file_storage();

    // Pre-populate tool cache
    {
        let mut cache: HashMap<String, String> = HashMap::new();
        cache.insert(
            "i-rs-kv".into(),
            serde_json::to_string(&serde_json::json!({
                "description": "KV store"
            }))
            .unwrap(),
        );
        storage.tool_cache.save("default", &cache).await.unwrap();
    }

    let session_id = create_session(&storage).await;

    let script = Script {
        meta: ScriptMeta {
            tool: "i-rs-kv".into(),
            name: "verify_tool_cache".into(),
            description: "Verify tool cache state in storage".into(),
            required_capabilities: vec!["add".into()],
            storage_backends: vec!["file".into()],
            tags: vec!["storage-verify".into()],
        },
        steps: vec![ScriptStep {
            step: 1,
            title: "检查工具缓存".into(),
            user_message: "存一下".into(),
            expected_tool: Some("i-rs-kv".into()),
            expected_command: Some("add".into()),
            expected_args: Some(HashMap::from([("KEY".into(), "test".into())])),
            expected_flags: None,
            check_reply: None,
            verify_storage: Some(StorageCheck {
                file_check: None,
                expected_state: Some(serde_json::json!({
                    "tool_cache": { "agent": "default", "length": 1 },
                })),
                no_new_records: None,
            }),
        }],
    };

    let mut session = MockSession::new(vec![StepOutput {
        reply: "已记录".into(),
        tool_calls: vec![
            ToolCallInfo::new("i-rs-kv")
                .with_command("add")
                .with_arg("KEY", "test")
                .with_arg("VALUE", "value"),
        ],
        prompt_tokens: 100,
        completion_tokens: 30,
        session_id: session_id.clone(),
    }]);

    let runner = ScriptRunner::new(script);
    let result = runner.run_with_storage(&mut session, Some(&storage)).await;

    assert!(result.passed, "tool cache verification should pass");
}

#[tokio::test]
async fn test_storage_verification_file_exists() {
    let (storage, dir) = file_storage();
    let session_id = create_session(&storage).await;
    let log_path = dir
        .path()
        .join("claw")
        .join("sessions")
        .join(format!("{}.jsonl", session_id));

    let script = Script {
        meta: ScriptMeta {
            tool: "i-rs-kv".into(),
            name: "verify_file_exists".into(),
            description: "Verify storage files exist on disk".into(),
            required_capabilities: vec!["add".into()],
            storage_backends: vec!["file".into()],
            tags: vec!["storage-verify".into()],
        },
        steps: vec![ScriptStep {
            step: 1,
            title: "检查文件".into(),
            user_message: "存一下".into(),
            expected_tool: Some("i-rs-kv".into()),
            expected_command: Some("add".into()),
            expected_args: Some(HashMap::from([("KEY".into(), "test".into())])),
            expected_flags: None,
            check_reply: None,
            verify_storage: Some(StorageCheck {
                file_check: Some(log_path.to_string_lossy().to_string()),
                expected_state: None,
                no_new_records: None,
            }),
        }],
    };

    // Create the message file to satisfy the file check
    {
        let msg = Message::User {
            text: "test".into(),
        };
        storage
            .message_log
            .append_one(&session_id, &msg)
            .await
            .unwrap();
    }

    let mut session = MockSession::new(vec![StepOutput {
        reply: "已记录".into(),
        tool_calls: vec![
            ToolCallInfo::new("i-rs-kv")
                .with_command("add")
                .with_arg("KEY", "test")
                .with_arg("VALUE", "val"),
        ],
        prompt_tokens: 100,
        completion_tokens: 30,
        session_id: session_id.clone(),
    }]);

    let runner = ScriptRunner::new(script);
    let result = runner.run_with_storage(&mut session, Some(&storage)).await;

    assert!(result.passed, "file existence check should pass");
    assert!(
        result.steps[0]
            .storage_results
            .iter()
            .any(|c| c.check_type == "file_check" && c.passed)
    );
}
