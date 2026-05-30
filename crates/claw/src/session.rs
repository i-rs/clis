use crate::utils::atomic_write;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
        matches!(self, SessionState::WaitingForTool | SessionState::WaitingForApproval)
    }
}

/// Metadata for a saved conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    #[serde(default = "default_agent_id")]
    pub agent_id: String,
    #[serde(default)]
    pub state: SessionState,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
}

fn default_agent_id() -> String {
    "default".to_string()
}

pub struct SessionManager {
    claw_dir: PathBuf,
    sessions: Vec<SessionMeta>,
    current_id: Option<String>,
}

impl SessionManager {
    pub fn new(claw_dir: PathBuf) -> Self {
        let sessions = Self::load_index(&claw_dir);
        let current_id = sessions.first().map(|s| s.id.clone());
        Self { claw_dir, sessions, current_id }
    }

    pub fn sessions(&self) -> &[SessionMeta] { &self.sessions }
    pub fn current_id(&self) -> Option<&str> { self.current_id.as_deref() }

    pub fn current_session(&self) -> Option<&SessionMeta> {
        self.current_id
            .as_ref()
            .and_then(|id| self.sessions.iter().find(|s| s.id == id.as_str()))
    }

    pub fn switch_to(&mut self, id: &str) -> bool {
        if self.sessions.iter().any(|s| s.id == id) {
            self.current_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn create_session(&mut self) -> String { self.create_session_for("default") }

    pub fn create_session_for(&mut self, agent_id: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_secs();
        self.sessions.push(SessionMeta {
            id: id.clone(),
            title: "新对话".to_string(),
            agent_id: agent_id.to_string(),
            state: SessionState::Active,
            created_at: now,
            updated_at: now,
            message_count: 0,
        });
        self.current_id = Some(id.clone());
        self.save_index();
        id
    }

    #[allow(dead_code)]
    pub fn delete_session(&mut self, id: &str) -> bool {
        let pos = self.sessions.iter().position(|s| s.id == id);
        if let Some(idx) = pos {
            self.sessions.remove(idx);
            let _ = std::fs::remove_file(self.messages_path(id));
            let _ = std::fs::remove_file(self.api_cache_path(id));
            if self.current_id.as_deref() == Some(id) {
                self.current_id = self.sessions.first().map(|s| s.id.clone());
            }
            self.save_index();
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn session_meta(&self, id: &str) -> Option<&SessionMeta> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn transition_state(&mut self, id: &str, new_state: SessionState) -> bool {
        if let Some(meta) = self.sessions.iter_mut().find(|s| s.id == id) {
            if meta.state.can_transition_to(&new_state) {
                meta.state = new_state;
                meta.updated_at = now_secs();
                self.save_index();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn mark_waiting_for_tool(&mut self, id: &str) -> bool { self.transition_state(id, SessionState::WaitingForTool) }
    #[allow(dead_code)]
    pub fn mark_waiting_for_approval(&mut self, id: &str) -> bool { self.transition_state(id, SessionState::WaitingForApproval) }
    pub fn mark_active(&mut self, id: &str) -> bool { self.transition_state(id, SessionState::Active) }
    #[allow(dead_code)]
    pub fn mark_completed(&mut self, id: &str) -> bool { self.transition_state(id, SessionState::Completed) }
    #[allow(dead_code)]
    pub fn mark_interrupted(&mut self, id: &str) -> bool { self.transition_state(id, SessionState::Interrupted) }
    pub fn mark_error(&mut self, id: &str, msg: &str) -> bool { self.transition_state(id, SessionState::Error(msg.to_string())) }
    #[allow(dead_code)]
    pub fn sessions_by_state(&self, state: &SessionState) -> Vec<&SessionMeta> { self.sessions.iter().filter(|s| &s.state == state).collect() }

    #[allow(dead_code)]
    pub fn save_plan_steps(&self, id: &str, steps: &[crate::app::PlanStep]) {
        let path = self.plan_steps_path(id);
        if let Ok(content) = serde_json::to_string(steps)
            && let Err(e) = atomic_write(&path, &content) { tracing::error!("持久化写入失败: {}", e); }
    }

    #[allow(dead_code)]
    pub fn load_plan_steps(&self, id: &str) -> Vec<crate::app::PlanStep> {
        let path = self.plan_steps_path(id);
        if !path.exists() { return Vec::new(); }
        if let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(steps) = serde_json::from_str(&content) { return steps; }
        Vec::new()
    }

    #[allow(dead_code)]
    pub fn search_sessions(&self, query: &str) -> Vec<&SessionMeta> {
        if query.is_empty() { return self.sessions.iter().collect(); }
        let q = query.to_lowercase();
        self.sessions.iter().filter(|s| s.title.to_lowercase().contains(&q)).collect()
    }

    pub fn export_markdown(&self, id: &str) -> Option<String> {
        let records = self.load_messages(id, 1000);
        let meta = self.sessions.iter().find(|s| s.id == id)?;
        let mut md = format!("# 会话：{}\n\n> 创建时间：{}\n\n", meta.title,
            chrono::DateTime::from_timestamp(meta.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_default());
        for record in &records {
            let msg_type = record.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let text = record.get("text").and_then(|t| t.as_str()).unwrap_or("");
            match msg_type {
                "user" => md.push_str(&format!("**用户:** {}\n\n", text)),
                "assistant" => md.push_str(&format!("**Claw:** {}\n\n", text)),
                "tool_call" => md.push_str(&format!("*[工具调用: {}]*\n\n",
                    record.get("name").and_then(|n| n.as_str()).unwrap_or(""))),
                "error" => md.push_str(&format!("**错误:** {}\n\n", text)),
                "evaluation" => {
                    let tool_name = record.get("tool").and_then(|v| v.as_str()).unwrap_or("");
                    let valid = record.get("valid").and_then(|v| v.as_bool()).unwrap_or(true);
                    let issues: Vec<String> = record.get("issues").and_then(|i| i.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                    if !valid {
                        md.push_str(&format!("**评测 ({}):** {}\n\n", tool_name, issues.join("; ")));
                    }
                }
                _ => {}
            }
        }
        Some(md)
    }

    pub fn export_json(&self, id: &str) -> Option<String> {
        let records = self.load_messages(id, 1000);
        let meta = self.sessions.iter().find(|s| s.id == id)?;
        let export = serde_json::json!({
            "session": { "id": meta.id, "title": meta.title, "created_at": meta.created_at, "updated_at": meta.updated_at },
            "messages": records,
        });
        serde_json::to_string_pretty(&export).ok()
    }

    pub fn rename_session(&mut self, id: &str, title: &str) -> bool {
        if let Some(meta) = self.sessions.iter_mut().find(|s| s.id == id) {
            meta.title = title.to_string();
            self.save_index();
            true
        } else { false }
    }

    pub fn append_message(&mut self, role: &str, content: &str, extra: Option<serde_json::Value>) {
        let session_id = match self.ensure_current_session() { Some(id) => id, None => return };
        let mut entry = serde_json::json!({"type": role, "text": content});
        if let Some(extra) = extra
            && let Some(obj) = entry.as_object_mut()
                && let Some(extra_obj) = extra.as_object() {
                    for (k, v) in extra_obj { obj.insert(k.clone(), v.clone()); }
                }
        let path = self.messages_path(&session_id);
        if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
        let line = match serde_json::to_string(&entry) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("序列化消息失败: {}", e);
                return;
            }
        };
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            use std::io::Write;
            if let Err(e) = writeln!(file, "{}", line) {
                tracing::error!("写入会话消息失败 ({}): {}", path.display(), e);
            }
        } else {
            tracing::error!("无法打开会话文件: {}", path.display());
        }
        if let Some(meta) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            meta.message_count += 1;
            meta.updated_at = now_secs();
        }
        self.save_index();
    }

    pub fn load_messages(&self, id: &str, max_messages: usize) -> Vec<serde_json::Value> {
        let path = self.messages_path(id);
        if !path.exists() { return Vec::new(); }
        let content = match std::fs::read_to_string(&path) { Ok(c) => c, Err(_) => return Vec::new() };
        let all_lines: Vec<serde_json::Value> = content
            .lines().filter_map(|line| {
                if line.trim().is_empty() { None } else { serde_json::from_str(line).ok() }
            }).collect();
        if all_lines.len() > max_messages { all_lines[all_lines.len() - max_messages..].to_vec() } else { all_lines }
    }

    pub fn load_app_messages(&self, id: &str, max_messages: usize) -> Vec<crate::app::Message> {
        self.load_messages(id, max_messages).into_iter().filter_map(|v| {
            crate::app::message_from_jsonl(&v)
        }).collect()
    }

    pub fn save_all_messages(&self, id: &str, records: &[serde_json::Value]) {
        let path = self.messages_path(id);
        let content: String = records
            .iter()
            .filter_map(|record| {
                serde_json::to_string(record).ok().map(|line| line + "\n")
            })
            .collect();
        if let Err(e) = atomic_write(&path, &content) { tracing::error!("持久化写入失败: {}", e); }
    }

    pub fn save_api_messages(&self, id: &str, messages: &[serde_json::Value]) {
        let path = self.api_cache_path(id);
        if let Ok(content) = serde_json::to_string(messages)
            && let Err(e) = atomic_write(&path, &content) { tracing::error!("持久化写入失败: {}", e); }
    }

    pub fn load_api_messages(&self, id: &str) -> Option<Vec<serde_json::Value>> {
        let path = self.api_cache_path(id);
        if !path.exists() { return None; }
        if let Ok(content) = std::fs::read_to_string(&path) { serde_json::from_str(&content).ok() } else { None }
    }

    fn ensure_current_session(&mut self) -> Option<String> {
        if self.current_id.is_some() { self.current_id.clone() } else { Some(self.create_session()) }
    }

    fn messages_path(&self, id: &str) -> PathBuf { self.claw_dir.join("sessions").join(format!("{}.jsonl", id)) }
    fn api_cache_path(&self, id: &str) -> PathBuf { self.claw_dir.join("sessions").join(format!("{}_api.json", id)) }
    fn plan_steps_path(&self, id: &str) -> PathBuf { self.claw_dir.join("sessions").join(format!("{}_plan.json", id)) }
    fn index_path(claw_dir: &Path) -> PathBuf { claw_dir.join("index.json") }
    
    fn load_index(claw_dir: &Path) -> Vec<SessionMeta> {
        let path = Self::index_path(claw_dir);
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(sessions) = serde_json::from_str(&content) { return sessions; }
        Vec::new()
    }

    fn save_index(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.sessions)
            && let Err(e) = atomic_write(&Self::index_path(&self.claw_dir), &content) { tracing::error!("持久化写入失败: {}", e); }
    }
}

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
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
        let msgs = vec![serde_json::json!({"type": "user", "text": "hello"})];
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
        mgr.save_plan_steps(&id, &[]);
        let loaded = mgr.load_plan_steps(&id);
        assert!(loaded.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_state_transitions() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone());
        let id = mgr.create_session();
        assert_eq!(mgr.session_meta(&id).unwrap().state, SessionState::Active);
        assert!(mgr.mark_waiting_for_tool(&id));
        assert_eq!(mgr.session_meta(&id).unwrap().state, SessionState::WaitingForTool);
        assert!(mgr.mark_active(&id));
        assert_eq!(mgr.session_meta(&id).unwrap().state, SessionState::Active);
        assert!(mgr.mark_completed(&id));
        assert!(!mgr.mark_active(&id));
        assert_eq!(mgr.session_meta(&id).unwrap().state, SessionState::Completed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_state_filters() {
        let dir = test_dir();
        let mut mgr = SessionManager::new(dir.clone());
        mgr.create_session_for("agent_a");
        let id2 = mgr.create_session_for("agent_b");
        mgr.mark_completed(&id2);
        let completed = mgr.sessions_by_state(&SessionState::Completed);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].agent_id, "agent_b");
        let active = mgr.sessions_by_state(&SessionState::Active);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].agent_id, "agent_a");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
