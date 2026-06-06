use crate::storage::ClawStorage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Lifecycle state of a conversation session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SessionState {
    #[default]
    Active,
    WaitingForTool,
    WaitingForApproval,
    #[serde(skip_serializing, deserialize_with = "deserialize_error_state")]
    Error(String),
    Completed,
    Interrupted,
}

fn deserialize_error_state<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(String::deserialize(deserializer).unwrap_or_default())
}

impl SessionState {
    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            SessionState::Active => "活跃",
            SessionState::WaitingForTool => "等待工具",
            SessionState::WaitingForApproval => "等待确认",
            SessionState::Error(_) => "错误",
            SessionState::Completed => "已完成",
            SessionState::Interrupted => "已中断",
        }
    }

    pub fn can_transition_to(&self, target: &SessionState) -> bool {
        use SessionState::*;
        match (self, target) {
            (Active, _) => true,
            (WaitingForTool, Active) => true,
            (WaitingForTool, Error(_)) => true,
            (WaitingForTool, Interrupted) => true,
            (WaitingForApproval, Active) => true,
            (WaitingForApproval, Error(_)) => true,
            (WaitingForApproval, Interrupted) => true,
            (Error(_), Active) => true,
            (Error(_), Completed) => true,
            (Completed, Active) => false,
            (Interrupted, Active) => false,
            (Completed, Completed) => true,
            (Interrupted, Interrupted) => true,
            _ if self == target => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn is_recoverable(&self) -> bool {
        matches!(self, SessionState::Error(_) | SessionState::Active)
    }

    #[allow(dead_code)]
    pub fn is_terminal(&self) -> bool {
        matches!(self, SessionState::Completed | SessionState::Interrupted)
    }

    #[allow(dead_code)]
    pub fn is_waiting(&self) -> bool {
        matches!(
            self,
            SessionState::WaitingForTool | SessionState::WaitingForApproval
        )
    }
}

/// Metadata for a saved conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct SessionMeta {
    pub id: String,
    pub title: String,
    #[serde(default = "default_agent_id")]
    pub agent_id: String,
    #[serde(default = "default_user_id")]
    pub user_id: String,
    #[serde(default)]
    pub state: SessionState,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
}

fn default_agent_id() -> String {
    "default".to_string()
}

fn default_user_id() -> String {
    "default".to_string()
}

pub struct SessionManager {
    storage: Arc<ClawStorage>,
    sessions: Vec<SessionMeta>,
    index: HashMap<String, usize>,
    current_id: Option<String>,
    /// Per-session count of messages already persisted to MessageLog.
    /// Used by TUI to append only the new tail on save.
    saved_cursors: HashMap<String, usize>,
}

impl SessionManager {
    /// Create a new SessionManager backed by the file storage (backward-compatible).
    pub fn new(claw_dir: PathBuf) -> anyhow::Result<Self> {
        let storage = Arc::new(ClawStorage::file(claw_dir));
        Self::with_storage(storage)
    }

    /// Create a SessionManager with a custom storage backend (for DI/testing).
    pub fn with_storage(storage: Arc<ClawStorage>) -> anyhow::Result<Self> {
        let sessions =
            crate::utils::sync_block_on(async { storage.sessions.load_all().await })
                .inspect_err(|e| {
                    tracing::error!(
                        "加载会话列表失败: {} — 数据可能损坏，请检查 .bak 备份后重新启动",
                        e
                    );
                })?;
        let current_id = sessions.first().map(|s| s.id.clone());
        let index = sessions
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.clone(), i))
            .collect();
        Ok(Self {
            storage,
            sessions,
            index,
            current_id,
            saved_cursors: HashMap::new(),
        })
    }

    pub fn sessions(&self) -> &[SessionMeta] {
        &self.sessions
    }
    pub fn current_id(&self) -> Option<&str> {
        self.current_id.as_deref()
    }

    #[inline]
    fn find_index(&self, id: &str) -> Option<usize> {
        self.index.get(id).copied()
    }

    fn rebuild_index(&mut self) {
        self.index = self
            .sessions
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.clone(), i))
            .collect();
    }

    pub fn current_session(&self) -> Option<&SessionMeta> {
        self.current_id
            .as_ref()
            .and_then(|id| self.find_index(id).map(|i| &self.sessions[i]))
    }

    pub fn switch_to(&mut self, id: &str) -> bool {
        if self.find_index(id).is_some() {
            self.current_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn create_session(&mut self) -> String {
        self.create_session_for("default", "default")
    }

    pub fn create_session_for(&mut self, agent_id: &str, user_id: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_secs();
        let idx = self.sessions.len();
        self.sessions.push(SessionMeta {
            id: id.clone(),
            title: "新对话".to_string(),
            agent_id: agent_id.to_string(),
            user_id: user_id.to_string(),
            state: SessionState::Active,
            created_at: now,
            updated_at: now,
            message_count: 0,
        });
        self.index.insert(id.clone(), idx);
        self.current_id = Some(id.clone());
        self.save_session(&id);
        id
    }

    #[allow(dead_code)]
    pub fn delete_session(&mut self, id: &str) -> bool {
        if let Some(idx) = self.find_index(id) {
            self.sessions.remove(idx);
            self.rebuild_index();
            let storage = self.storage.clone();
            let sid = id.to_string();
            crate::utils::sync_block_on(async move {
                let _ = storage.sessions.delete_one(&sid).await;
                let _ = storage.message_log.delete_session(&sid).await;
                let _ = storage.api_cache.delete(&sid).await;
                let _ = storage.plan_steps.delete(&sid).await;
            });
            if self.current_id.as_deref() == Some(id) {
                self.current_id = self.sessions.first().map(|s| s.id.clone());
            }
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn session_meta(&self, id: &str) -> Option<&SessionMeta> {
        self.find_index(id).map(|i| &self.sessions[i])
    }

    pub fn transition_state(&mut self, id: &str, new_state: SessionState) -> bool {
        if let Some(idx) = self.find_index(id) {
            let meta = &mut self.sessions[idx];
            if meta.state.can_transition_to(&new_state) {
                meta.state = new_state;
                meta.updated_at = now_secs();
                self.save_session(id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn mark_waiting_for_tool(&mut self, id: &str) -> bool {
        self.transition_state(id, SessionState::WaitingForTool)
    }
    #[allow(dead_code)]
    pub fn mark_waiting_for_approval(&mut self, id: &str) -> bool {
        self.transition_state(id, SessionState::WaitingForApproval)
    }
    pub fn mark_active(&mut self, id: &str) -> bool {
        self.transition_state(id, SessionState::Active)
    }
    #[allow(dead_code)]
    pub fn mark_completed(&mut self, id: &str) -> bool {
        self.transition_state(id, SessionState::Completed)
    }
    #[allow(dead_code)]
    pub fn mark_interrupted(&mut self, id: &str) -> bool {
        self.transition_state(id, SessionState::Interrupted)
    }
    pub fn mark_error(&mut self, id: &str, msg: &str) -> bool {
        self.transition_state(id, SessionState::Error(msg.to_string()))
    }
    #[allow(dead_code)]
    pub fn sessions_by_state(&self, state: &SessionState) -> Vec<&SessionMeta> {
        self.sessions.iter().filter(|s| &s.state == state).collect()
    }

    #[allow(dead_code)]
    pub fn save_plan_steps(&self, id: &str, steps: &[crate::app::PlanStep]) {
        let storage = self.storage.clone();
        let sid = id.to_string();
        let steps = steps.to_vec();
        if let Err(e) =
            crate::utils::sync_block_on(async move { storage.plan_steps.save(&sid, &steps).await })
        {
            tracing::error!("持久化写入失败: {}", e);
        }
    }

    #[allow(dead_code)]
    pub fn load_plan_steps(&self, id: &str) -> Vec<crate::app::PlanStep> {
        let storage = self.storage.clone();
        let sid = id.to_string();
        crate::utils::sync_block_on(async move { storage.plan_steps.load(&sid).await })
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn search_sessions(&self, query: &str) -> Vec<&SessionMeta> {
        if query.is_empty() {
            return self.sessions.iter().collect();
        }
        let q = query.to_lowercase();
        self.sessions
            .iter()
            .filter(|s| s.title.to_lowercase().contains(&q))
            .collect()
    }

    pub fn export_markdown(&self, id: &str) -> Option<String> {
        let messages = self.load_app_messages(id, 1000);
        let meta = self.find_index(id).map(|i| &self.sessions[i])?;
        let mut md = format!(
            "# 会话：{}\n\n> 创建时间：{}\n\n",
            meta.title,
            chrono::DateTime::from_timestamp(meta.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default()
        );
        for msg in &messages {
            match msg {
                crate::app::Message::User { text } => {
                    md.push_str(&format!("**用户:** {}\n\n", text));
                }
                crate::app::Message::Assistant { text, .. } => {
                    md.push_str(&format!("**Claw:** {}\n\n", text));
                }
                crate::app::Message::ToolCall { name, .. } => {
                    md.push_str(&format!("*[工具调用: {}]*\n\n", name));
                }
                crate::app::Message::Error { text } => {
                    md.push_str(&format!("**错误:** {}\n\n", text));
                }
                crate::app::Message::Evaluation {
                    tool,
                    valid,
                    issues,
                } if !valid => {
                    md.push_str(&format!("**评测 ({}):** {}\n\n", tool, issues.join("; ")));
                }
                _ => {}
            }
        }
        Some(md)
    }

    pub fn export_json(&self, id: &str) -> Option<String> {
        let messages = self.load_app_messages(id, 1000);
        let meta = self.find_index(id).map(|i| &self.sessions[i])?;
        let export = serde_json::json!({
            "session": { "id": meta.id, "title": meta.title, "created_at": meta.created_at, "updated_at": meta.updated_at },
            "messages": messages,
        });
        serde_json::to_string_pretty(&export).ok()
    }

    pub fn rename_session(&mut self, id: &str, title: &str) -> bool {
        if let Some(idx) = self.find_index(id) {
            self.sessions[idx].title = title.to_string();
            self.save_session(id);
            true
        } else {
            false
        }
    }

    pub fn load_app_messages(&self, id: &str, max_messages: usize) -> Vec<crate::app::Message> {
        let log = self.storage.message_log.clone();
        let sid = id.to_string();
        crate::utils::sync_block_on(async move { log.load(&sid, max_messages).await })
            .unwrap_or_default()
    }

    /// Get a clonable handle to the append-only MessageLog.
    #[allow(dead_code)]
    pub fn message_log(&self) -> std::sync::Arc<dyn crate::storage::MessageLog> {
        self.storage.message_log.clone()
    }

    /// Persist messages to the message log for a session. Unlike
    /// `append_new_messages`, this does not maintain a cursor — it is
    /// intended for external callers (gateway, dashboard) that write
    /// complete message pairs directly.
    ///
    /// Updates `message_count` and `updated_at` on the in-memory session
    /// metadata, then saves via `save_session`.
    pub fn persist_messages(
        &mut self,
        session_id: &str,
        messages: &[crate::app::Message],
    ) -> anyhow::Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        let log = self.storage.message_log.clone();
        let sid = session_id.to_string();
        let append_count = messages.len();
        let msgs = messages.to_vec();
        crate::utils::sync_block_on(async move { log.append_batch(&sid, &msgs).await })?;

        if let Some(idx) = self.index.get(session_id)
            && let Some(meta) = self.sessions.get_mut(*idx)
        {
            meta.message_count += append_count;
            meta.updated_at = chrono::Utc::now().timestamp();
        }
        self.save_session(session_id);
        Ok(())
    }

    /// Append any messages in `messages` beyond the saved cursor to the
    /// MessageLog. Updates the cursor on success.
    pub fn append_new_messages(&mut self, session_id: &str, messages: &[crate::app::Message]) {
        let cursor = self.saved_cursors.get(session_id).copied().unwrap_or(0);
        if cursor >= messages.len() {
            return;
        }
        let new_msgs = messages[cursor..].to_vec();
        let log = self.storage.message_log.clone();
        let sid = session_id.to_string();
        let append_count = new_msgs.len();
        let result =
            crate::utils::sync_block_on(async move { log.append_batch(&sid, &new_msgs).await });
        match result {
            Ok(()) => {
                self.saved_cursors
                    .insert(session_id.to_string(), messages.len());
                // 更新 SessionMeta.message_count
                if append_count > 0 {
                    if let Some(idx) = self.index.get(session_id)
                        && let Some(meta) = self.sessions.get_mut(*idx)
                    {
                        meta.message_count += append_count;
                        meta.updated_at = chrono::Utc::now().timestamp();
                    }
                    self.save_session(session_id);
                }
            }
            Err(e) => tracing::error!("append_new_messages 失败: {}", e),
        }
    }

    /// Reset the cursor when loading a session (so subsequent appends start
    /// from the loaded count).
    pub fn reset_cursor(&mut self, session_id: &str, count: usize) {
        self.saved_cursors.insert(session_id.to_string(), count);
    }

    pub fn save_api_messages(&self, id: &str, messages: &[serde_json::Value]) {
        let storage = self.storage.clone();
        let sid = id.to_string();
        let messages = messages.to_vec();
        if let Err(e) =
            crate::utils::sync_block_on(
                async move { storage.api_cache.save(&sid, &messages).await },
            )
        {
            tracing::error!("持久化写入失败: {}", e);
        }
    }

    pub fn load_api_messages(&self, id: &str) -> Option<Vec<serde_json::Value>> {
        let storage = self.storage.clone();
        let sid = id.to_string();
        crate::utils::sync_block_on(async move { storage.api_cache.load(&sid).await })
            .unwrap_or(None)
    }

    #[allow(dead_code)]
    fn ensure_current_session(&mut self) -> Option<String> {
        if self.current_id.is_some() {
            self.current_id.clone()
        } else {
            Some(self.create_session())
        }
    }

    /// Flush all sessions to storage (bulk save). Use only for shutdown/migration.
    /// Prefer `save_session` for single-session updates.
    #[deprecated(note = "use save_session for single-session updates")]
    pub fn save_index(&self) {
        let storage = self.storage.clone();
        let sessions = self.sessions.clone();
        if let Err(e) =
            crate::utils::sync_block_on(async move { storage.sessions.save_all(&sessions).await })
        {
            tracing::error!("持久化写入失败: {}", e);
        }
    }

    /// Upsert a single session to storage (avoids rewriting the entire index).
    fn save_session(&self, id: &str) {
        let Some(idx) = self.find_index(id) else {
            tracing::warn!("save_session: session '{}' 不在 index 中，保存已跳过", id);
            return;
        };
        let storage = self.storage.clone();
        let meta = self.sessions[idx].clone();
        if let Err(e) =
            crate::utils::sync_block_on(async move { storage.sessions.upsert(&meta).await })
        {
            tracing::error!("持久化写入失败: {}", e);
        }
    }
}

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir() -> PathBuf {
        let id = uuid::Uuid::new_v4().to_string();
        let dir = std::env::temp_dir().join(format!("i-rs-claw-test-{}", id));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_create_session() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        assert!(mgr.current_id().is_some());
        assert_eq!(mgr.sessions().len(), 1);
        assert!(mgr.current_session().is_some());
        assert!(!id.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_load_messages() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        let msgs = vec![
            crate::app::Message::User {
                text: "hello".into(),
            },
            crate::app::Message::Assistant {
                text: "hi there".into(),
                reasoning: String::new(),
                token_usage: None,
            },
        ];
        mgr.append_new_messages(&id, &msgs);
        let loaded = mgr.load_app_messages(&id, 10);
        assert_eq!(loaded.len(), 2);
        match &loaded[0] {
            crate::app::Message::User { text } => assert_eq!(text, "hello"),
            other => panic!("expected User, got {:?}", other),
        }
        match &loaded[1] {
            crate::app::Message::Assistant { text, .. } => assert_eq!(text, "hi there"),
            other => panic!("expected Assistant, got {:?}", other),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_load_api_messages() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        let msgs = vec![serde_json::json!({"role": "user", "content": "hello"})];
        mgr.save_api_messages(&id, &msgs);
        let loaded = mgr.load_api_messages(&id);
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rename_session() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        mgr.rename_session(&id, "My Chat");
        assert_eq!(mgr.current_session().unwrap().title, "My Chat");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_sessions() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id1 = mgr.create_session();
        mgr.rename_session(&id1, "Weather Talk");
        let id2 = mgr.create_session();
        mgr.rename_session(&id2, "Coding Help");
        let results = mgr.search_sessions("weather");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Weather Talk");
        let all = mgr.search_sessions("");
        assert_eq!(all.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_markdown() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        mgr.rename_session(&id, "Test Chat");
        let msgs = vec![
            crate::app::Message::User {
                text: "hello".into(),
            },
            crate::app::Message::Assistant {
                text: "hi there".into(),
                reasoning: String::new(),
                token_usage: None,
            },
        ];
        mgr.append_new_messages(&id, &msgs);
        let md = mgr.export_markdown(&id);
        assert!(md.is_some());
        let md = md.unwrap();
        assert!(md.contains("Test Chat"));
        assert!(md.contains("hello"));
        assert!(md.contains("hi there"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_json() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        let msgs = vec![crate::app::Message::User {
            text: "hello".into(),
        }];
        mgr.append_new_messages(&id, &msgs);
        let json = mgr.export_json(&id);
        assert!(json.is_some());
        let parsed: serde_json::Value = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(parsed["session"]["id"], id);
        assert_eq!(parsed["messages"].as_array().unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_plan_steps_persistence() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        let steps = vec![
            crate::app::PlanStep {
                description: "Step 1".to_string(),
                done: false,
            },
            crate::app::PlanStep {
                description: "Step 2".to_string(),
                done: true,
            },
        ];
        mgr.save_plan_steps(&id, &steps);
        let loaded = mgr.load_plan_steps(&id);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].description, "Step 1");
        assert!(!loaded[0].done);
        assert!(loaded[1].done);
        mgr.save_plan_steps(&id, &[]);
        let loaded = mgr.load_plan_steps(&id);
        assert!(loaded.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_state_transitions() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        let id = mgr.create_session();
        assert_eq!(mgr.session_meta(&id).unwrap().state, SessionState::Active);
        assert!(mgr.mark_waiting_for_tool(&id));
        assert_eq!(
            mgr.session_meta(&id).unwrap().state,
            SessionState::WaitingForTool
        );
        assert!(mgr.mark_active(&id));
        assert_eq!(mgr.session_meta(&id).unwrap().state, SessionState::Active);
        assert!(mgr.mark_completed(&id));
        assert!(!mgr.mark_active(&id));
        assert_eq!(
            mgr.session_meta(&id).unwrap().state,
            SessionState::Completed
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_state_filters() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone()).unwrap();
        mgr.create_session_for("agent_a", "default");
        let id2 = mgr.create_session_for("agent_b", "default");
        mgr.mark_completed(&id2);
        let completed = mgr.sessions_by_state(&SessionState::Completed);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].agent_id, "agent_b");
        let active = mgr.sessions_by_state(&SessionState::Active);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].agent_id, "agent_a");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_create_does_not_wipe_messages() {
        let path = std::env::temp_dir().join(format!("i-rs-claw-test-{}.db", uuid::Uuid::new_v4()));
        let _ = std::fs::remove_file(&path);
        let storage = std::sync::Arc::new(
            crate::utils::sync_block_on(crate::storage::ClawStorage::sqlite(path.clone())).unwrap(),
        );

        let mut mgr = SessionManager::with_storage(storage.clone()).unwrap();
        let id_a = mgr.create_session();
        let msgs = vec![
            crate::app::Message::User {
                text: "hello".into(),
            },
            crate::app::Message::Assistant {
                text: "hi there".into(),
                reasoning: String::new(),
                token_usage: None,
            },
        ];
        mgr.append_new_messages(&id_a, &msgs);

        let loaded = mgr.load_app_messages(&id_a, 100);
        assert_eq!(loaded.len(), 2, "before create_session");

        let _id_b = mgr.create_session();

        let mgr2 = SessionManager::with_storage(storage).unwrap();
        let loaded2 = mgr2.load_app_messages(&id_a, 100);
        assert_eq!(
            loaded2.len(),
            2,
            "messages of A should survive create_session for B"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn test_sqlite_switch_session_preserves_messages() {
        let path = std::env::temp_dir().join(format!("i-rs-claw-test-{}.db", uuid::Uuid::new_v4()));
        let _ = std::fs::remove_file(&path);
        let storage = std::sync::Arc::new(
            crate::utils::sync_block_on(crate::storage::ClawStorage::sqlite(path.clone())).unwrap(),
        );

        let mut mgr = SessionManager::with_storage(storage.clone()).unwrap();
        let id_a = mgr.create_session();
        mgr.append_new_messages(
            &id_a,
            &[crate::app::Message::User {
                text: "msg in A".into(),
            }],
        );
        let id_b = mgr.create_session();
        mgr.append_new_messages(
            &id_b,
            &[crate::app::Message::User {
                text: "msg in B".into(),
            }],
        );

        // Save & reload — verify both sessions retain their messages
        let mgr2 = SessionManager::with_storage(storage.clone()).unwrap();
        let msgs_a = mgr2.load_app_messages(&id_a, 100);
        let msgs_b = mgr2.load_app_messages(&id_b, 100);
        assert_eq!(msgs_a.len(), 1, "session A messages preserved");
        assert_eq!(msgs_b.len(), 1, "session B messages preserved");

        let _ = std::fs::remove_file(&path);
    }
}
