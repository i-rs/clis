use super::ScriptStep;
use i_rs_claw_core::session::SessionManager;
use i_rs_claw_core::storage::ClawStorage;
use std::sync::Arc;

/// Result of a single storage check.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageCheckResult {
    pub check_type: String,
    pub passed: bool,
    pub detail: String,
}

/// Run all storage checks defined in a script step.
pub async fn verify_step(
    storage: &Arc<ClawStorage>,
    step: &ScriptStep,
    session_id: &str,
) -> Vec<StorageCheckResult> {
    let Some(ref check) = step.verify_storage else {
        return vec![];
    };

    let mut results = Vec::new();

    // check_trait: Verify storage state via trait methods
    if let Some(ref expected) = check.expected_state {
        match verify_trait_checks(storage, session_id, expected).await {
            Ok(msg) => results.push(StorageCheckResult {
                check_type: "trait_check".into(),
                passed: true,
                detail: msg,
            }),
            Err(e) => results.push(StorageCheckResult {
                check_type: "trait_check".into(),
                passed: false,
                detail: e,
            }),
        }
    }

    // check_file: Verify file system state
    if let Some(ref path) = check.file_check {
        let exists = std::path::Path::new(path).exists();
        results.push(StorageCheckResult {
            check_type: "file_check".into(),
            passed: exists,
            detail: if exists {
                format!("文件存在: {}", path)
            } else {
                format!("文件不存在: {}", path)
            },
        });
    }

    // no_new_records: Verify session state didn't exceed expected count
    if check.no_new_records.unwrap_or(false) {
        match verify_no_new_records(storage, session_id).await {
            Ok(msg) => results.push(StorageCheckResult {
                check_type: "no_new_records".into(),
                passed: true,
                detail: msg,
            }),
            Err(e) => results.push(StorageCheckResult {
                check_type: "no_new_records".into(),
                passed: false,
                detail: e,
            }),
        }
    }

    results
}

async fn verify_trait_checks(
    storage: &Arc<ClawStorage>,
    session_id: &str,
    expected: &serde_json::Value,
) -> Result<String, String> {
    let mut details = Vec::new();

    let mgr = SessionManager::with_storage(storage.clone())
        .map_err(|e| format!("无法创建 SessionManager: {}", e))?;

    // Verify session exists
    if let Some(sessions) = expected.get("session") {
        let session_list = mgr.sessions().to_vec();

        if let Some(expected_len) = sessions.get("length").and_then(|v| v.as_u64()) {
            if (session_list.len() as u64) < expected_len {
                return Err(format!(
                    "预期 {} 个会话，实际 {}",
                    expected_len,
                    session_list.len()
                ));
            }
            details.push(format!("会话数: {}/{}", session_list.len(), expected_len));
        }

        if sessions
            .get("exists")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            let has_session = session_list.iter().any(|s| s.id == session_id);
            if !has_session {
                return Err(format!("未找到会话 '{}'", session_id));
            }
            details.push("会话已创建".into());
        }
    }

    // Verify message log has entries
    if let Some(msgs) = expected.get("messages") {
        let msg_count = storage
            .message_log
            .count(session_id)
            .await
            .map_err(|e| format!("无法获取消息数: {}", e))?;

        if let Some(expected_min) = msgs.get("min").and_then(|v| v.as_u64()) {
            if (msg_count as u64) < expected_min {
                return Err(format!(
                    "预期至少 {} 条消息，实际 {}",
                    expected_min, msg_count
                ));
            }
            details.push(format!("消息数: {}", msg_count));
        }
    }

    // Verify a specific tool cache entry exists
    if let Some(tool_cache) = expected.get("tool_cache")
        && let Some(agent) = tool_cache.get("agent").and_then(|v| v.as_str())
    {
        let cache = storage
            .tool_cache
            .load(agent)
            .await
            .map_err(|e| format!("无法加载工具缓存: {}", e))?;
        if let Some(expected_len) = tool_cache.get("length").and_then(|v| v.as_u64()) {
            if (cache.len() as u64) < expected_len {
                return Err(format!(
                    "工具缓存预期 {} 项，实际 {}",
                    expected_len,
                    cache.len()
                ));
            }
            details.push(format!("工具缓存: {} 项", cache.len()));
        }
    }

    Ok(details.join(", "))
}

async fn verify_no_new_records(
    storage: &Arc<ClawStorage>,
    _session_id: &str,
) -> Result<String, String> {
    let mgr = SessionManager::with_storage(storage.clone())
        .map_err(|e| format!("无法创建 SessionManager: {}", e))?;
    let sessions = mgr.sessions();

    if sessions.len() > 1 {
        return Err(format!("预期最多 1 个会话，实际 {}", sessions.len()));
    }

    Ok("未创建意外记录".into())
}
