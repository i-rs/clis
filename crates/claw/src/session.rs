use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata for a saved conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    /// Which agent profile this session belongs to.
    /// Defaults to "default" for backward compatibility.
    #[serde(default = "default_agent_id")]
    pub agent_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
}

fn default_agent_id() -> String {
    "default".to_string()
}

/// Manages conversation sessions with JSONL persistence.
///
/// Layout:
///   ~/.i-rs-claw/claw/
///   ├── index.json              # Session list (Vec<SessionMeta>)
///   └── sessions/
///       ├── {session_id}.jsonl  # App messages in JSONL format
///       └── {session_id}.json   # Cached API messages for context continuity
pub struct SessionManager {
    /// Directory for all session data (~/.i-rs-claw/claw/)
    claw_dir: PathBuf,
    /// All known sessions
    sessions: Vec<SessionMeta>,
    /// Currently active session ID
    current_id: Option<String>,
}

impl SessionManager {
    /// Create or load session manager from disk.
    pub fn new(claw_dir: PathBuf) -> Self {
        let sessions = Self::load_index(&claw_dir);
        let current_id = sessions.first().map(|s| s.id.clone());

        Self {
            claw_dir,
            sessions,
            current_id,
        }
    }

    // =============================================
    // Session list
    // =============================================

    pub fn sessions(&self) -> &[SessionMeta] {
        &self.sessions
    }

    pub fn current_id(&self) -> Option<&str> {
        self.current_id.as_deref()
    }

    pub fn current_session(&self) -> Option<&SessionMeta> {
        self.current_id
            .as_ref()
            .and_then(|id| self.sessions.iter().find(|s| &s.id == id))
    }

    /// Switch to an existing session, loading its messages.
    pub fn switch_to(&mut self, id: &str) -> bool {
        if self.sessions.iter().any(|s| s.id == id) {
            self.current_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    // =============================================
    // Session CRUD
    // =============================================

    /// Create a new session and return its ID.
    pub fn create_session(&mut self) -> String {
        self.create_session_for("default")
    }

    /// Create a new session for a specific agent and return its ID.
    pub fn create_session_for(&mut self, agent_id: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_secs();
        let meta = SessionMeta {
            id: id.clone(),
            title: "新对话".to_string(),
            agent_id: agent_id.to_string(),
            created_at: now,
            updated_at: now,
            message_count: 0,
        };
        self.sessions.push(meta);
        self.current_id = Some(id.clone());
        self.save_index();
        id
    }

    /// Delete a session and its files.
    #[allow(dead_code)]
    pub fn delete_session(&mut self, id: &str) -> bool {
        let pos = self.sessions.iter().position(|s| s.id == id);
        if let Some(idx) = pos {
            self.sessions.remove(idx);
            // Remove session files
            let _ = std::fs::remove_file(self.messages_path(id));
            let _ = std::fs::remove_file(self.api_cache_path(id));
            // If current session was deleted, switch to first available
            if self.current_id.as_deref() == Some(id) {
                self.current_id = self.sessions.first().map(|s| s.id.clone());
            }
            self.save_index();
            true
        } else {
            false
        }
    }

    /// Get session metadata by ID.
    #[allow(dead_code)]
    pub fn session_meta(&self, id: &str) -> Option<&SessionMeta> {
        self.sessions.iter().find(|s| s.id == id)
    }

    /// Search sessions by title keyword (case-insensitive).
    #[allow(dead_code)]
    pub fn save_plan_steps(&self, id: &str, steps: &[crate::app::PlanStep]) {
        let path = self.plan_steps_path(id);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string(steps) {
            let _ = std::fs::write(&path, content);
        }
    }

    /// Load plan steps from disk.
    pub fn load_plan_steps(&self, id: &str) -> Vec<crate::app::PlanStep> {
        let path = self.plan_steps_path(id);
        if !path.exists() {
            return Vec::new();
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(steps) = serde_json::from_str(&content) {
                return steps;
            }
        }
        Vec::new()
    }

    /// Search sessions by title keyword (case-insensitive).
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

    /// Export session messages as Markdown.
    pub fn export_markdown(&self, id: &str) -> Option<String> {
        let records = self.load_messages(id, 1000);
        let meta = self.sessions.iter().find(|s| s.id == id)?;

        let mut md = format!("# 会话：{}\n\n", meta.title);
        md.push_str(&format!(
            "> 创建时间：{}\n\n",
            chrono::DateTime::from_timestamp(meta.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default()
        ));

        for record in &records {
            let msg_type = record.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let text = record.get("text").and_then(|t| t.as_str()).unwrap_or("");
            match msg_type {
                "user" => md.push_str(&format!("**用户:** {}\n\n", text)),
                "assistant" => md.push_str(&format!("**Claw:** {}\n\n", text)),
                "tool_call" => {
                    let name = record.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    md.push_str(&format!("*[工具调用: {}]*\n\n", name));
                }
                "error" => md.push_str(&format!("**错误:** {}\n\n", text)),
                _ => {}
            }
        }
        Some(md)
    }

    /// Export session messages as JSON.
    pub fn export_json(&self, id: &str) -> Option<String> {
        let records = self.load_messages(id, 1000);
        let meta = self.sessions.iter().find(|s| s.id == id)?;

        let export = serde_json::json!({
            "session": {
                "id": meta.id,
                "title": meta.title,
                "created_at": meta.created_at,
                "updated_at": meta.updated_at,
            },
            "messages": records,
        });
        serde_json::to_string_pretty(&export).ok()
    }

    /// Rename a session (typically set title to first user message).
    pub fn rename_session(&mut self, id: &str, title: &str) -> bool {
        if let Some(meta) = self.sessions.iter_mut().find(|s| s.id == id) {
            meta.title = title.to_string();
            self.save_index();
            true
        } else {
            false
        }
    }

    // =============================================
    // Message persistence (JSONL)
    // =============================================

    /// Append a message to the current session's JSONL file.
    pub fn append_message(
        &mut self,
        role: &str,
        content: &str,
        extra: Option<serde_json::Value>,
    ) {
        let session_id = match self.ensure_current_session() {
            Some(id) => id,
            None => return,
        };

        let mut entry = serde_json::json!({
            "type": role,
            "text": content,
        });
        if let Some(extra) = extra {
            if let Some(obj) = entry.as_object_mut() {
                if let Some(extra_obj) = extra.as_object() {
                    for (k, v) in extra_obj {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        let path = self.messages_path(&session_id);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let line = serde_json::to_string(&entry).unwrap_or_default();
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = writeln!(file, "{}", line);
        }

        // Update session metadata
        if let Some(meta) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            meta.message_count += 1;
            meta.updated_at = now_secs();
        }
        self.save_index();
    }

    /// Load messages from a session's JSONL file.
    /// Returns up to `max_messages` most recent entries.
    pub fn load_messages(&self, id: &str, max_messages: usize) -> Vec<serde_json::Value> {
        let path = self.messages_path(id);
        if !path.exists() {
            return Vec::new();
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let all_lines: Vec<serde_json::Value> = content
            .lines()
            .filter_map(|line| {
                if line.trim().is_empty() {
                    None
                } else {
                    serde_json::from_str(line).ok()
                }
            })
            .collect();

        // Take last N messages (sliding window)
        if all_lines.len() > max_messages {
            all_lines[all_lines.len() - max_messages..].to_vec()
        } else {
            all_lines
        }
    }

    /// Load messages and convert to app::Message for display.
    pub fn load_app_messages(
        &self,
        id: &str,
        max_messages: usize,
    ) -> Vec<crate::app::Message> {
        self.load_messages(id, max_messages)
            .into_iter()
            .filter_map(|v| {
                let msg_type = v.get("type").and_then(|t| t.as_str())?;
                match msg_type {
                    "user" => Some(crate::app::Message::User {
                        text: v.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                    }),
                    "assistant" => Some(crate::app::Message::Assistant {
                        text: v.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                    }),
                    "tool_call" => Some(crate::app::Message::ToolCall {
                        name: v.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                        args: v.get("args").and_then(|a| a.as_str()).unwrap_or("").to_string(),
                        result: v.get("result").and_then(|r| r.as_str()).unwrap_or("").to_string(),
                        step: 0,
                        total_steps: 0,
                    }),
                    "error" => Some(crate::app::Message::Error {
                        text: v.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                    }),
                    _ => None,
                }
            })
            .collect()
    }

    /// Save all messages as JSONL, replacing any existing content.
    /// Messages are serde_json::Value records with a "type" field.
    pub fn save_all_messages(&self, id: &str, records: &[serde_json::Value]) {
        let path = self.messages_path(id);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut file = match std::fs::File::create(&path) {
            Ok(f) => std::io::BufWriter::new(f),
            Err(_) => return,
        };

        use std::io::Write;
        for record in records {
            if let Ok(line) = serde_json::to_string(record) {
                let _ = writeln!(&mut file, "{}", line);
            }
        }
    }

    // =============================================
    // API message cache (context continuity)
    // =============================================

    /// Cache API messages for context continuity.
    pub fn save_api_messages(&self, id: &str, messages: &[serde_json::Value]) {
        let path = self.api_cache_path(id);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string(messages) {
            let _ = std::fs::write(&path, content);
        }
    }

    /// Load cached API messages.
    pub fn load_api_messages(&self, id: &str) -> Option<Vec<serde_json::Value>> {
        let path = self.api_cache_path(id);
        if !path.exists() {
            return None;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    // =============================================
    // Internal helpers
    // =============================================

    fn ensure_current_session(&mut self) -> Option<String> {
        if self.current_id.is_some() {
            return self.current_id.clone();
        }
        Some(self.create_session())
    }

    fn messages_path(&self, id: &str) -> PathBuf {
        self.claw_dir.join("sessions").join(format!("{}.jsonl", id))
    }

    fn api_cache_path(&self, id: &str) -> PathBuf {
        self.claw_dir.join("sessions").join(format!("{}_api.json", id))
    }

    fn plan_steps_path(&self, id: &str) -> PathBuf {
        self.claw_dir.join("sessions").join(format!("{}_plan.json", id))
    }

    fn index_path(claw_dir: &PathBuf) -> PathBuf {
        claw_dir.join("index.json")
    }

    fn load_index(claw_dir: &PathBuf) -> Vec<SessionMeta> {
        let path = Self::index_path(claw_dir);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(sessions) = serde_json::from_str(&content) {
                    return sessions;
                }
            }
        }
        Vec::new()
    }

    fn save_index(&self) {
        let path = Self::index_path(&self.claw_dir);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&self.sessions) {
            let _ = std::fs::write(&path, content);
        }
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
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
        let mut mgr = SessionManager::new(dir.clone());
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
        let mut mgr = SessionManager::new(dir.clone());
        let id = mgr.create_session();

        let msgs = vec![
            serde_json::json!({"type": "user", "text": "hello"}),
            serde_json::json!({"type": "assistant", "text": "hi there"}),
        ];
        mgr.save_all_messages(&id, &msgs);

        let loaded = mgr.load_messages(&id, 10);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0]["text"], "hello");
        assert_eq!(loaded[1]["text"], "hi there");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_load_api_messages() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone());
        let id = mgr.create_session();

        let msgs = vec![
            serde_json::json!({"role": "user", "content": "hello"}),
        ];
        mgr.save_api_messages(&id, &msgs);

        let loaded = mgr.load_api_messages(&id);
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rename_session() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone());
        let id = mgr.create_session();

        mgr.rename_session(&id, "My Chat");
        assert_eq!(mgr.current_session().unwrap().title, "My Chat");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_sessions() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone());
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
        let mut mgr = SessionManager::new(dir.clone());
        let id = mgr.create_session();
        mgr.rename_session(&id, "Test Chat");

        let msgs = vec![
            serde_json::json!({"type": "user", "text": "hello"}),
            serde_json::json!({"type": "assistant", "text": "hi there"}),
        ];
        mgr.save_all_messages(&id, &msgs);

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
        let mut mgr = SessionManager::new(dir.clone());
        let id = mgr.create_session();

        let msgs = vec![
            serde_json::json!({"type": "user", "text": "hello"}),
        ];
        mgr.save_all_messages(&id, &msgs);

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
        let mut mgr = SessionManager::new(dir.clone());
        let id = mgr.create_session();

        let steps = vec![
            crate::app::PlanStep { description: "Step 1".to_string(), done: false },
            crate::app::PlanStep { description: "Step 2".to_string(), done: true },
        ];
        mgr.save_plan_steps(&id, &steps);

        let loaded = mgr.load_plan_steps(&id);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].description, "Step 1");
        assert!(!loaded[0].done);
        assert!(loaded[1].done);

        // Clear plan
        mgr.save_plan_steps(&id, &[]);
        let loaded = mgr.load_plan_steps(&id);
        assert!(loaded.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
