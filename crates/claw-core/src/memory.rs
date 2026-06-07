use crate::storage::ClawStorage;
use crate::utils::atomic_write;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cross-session memory that tracks tool usage patterns and user preferences.
///
/// When constructed via `for_agent_with_storage()`, I/O is delegated to the
/// active storage backend. Otherwise falls back to the legacy JSON file path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSessionMemory {
    /// Tool name → usage count across all sessions
    tool_frequency: HashMap<String, usize>,
    /// Ordered list of frequently used tools (determined by analysis)
    hot_tools: Vec<String>,
    /// User preference statements
    #[serde(default)]
    preferences: Vec<String>,
    /// User's name (collected during onboarding)
    #[serde(default)]
    user_name: Option<String>,
    /// What the user calls the assistant (pet name / nickname)
    #[serde(default)]
    assistant_nickname: Option<String>,
    /// Free-form user info facts (habits, preferences, etc.)
    #[serde(default)]
    user_info: Vec<String>,
    /// Session feedback: session_id -> (positive_count, negative_count)
    #[serde(default)]
    session_feedback: HashMap<String, (u32, u32)>,
    /// Path to disk cache file (legacy fallback)
    #[serde(skip)]
    path: PathBuf,
    /// Whether there are unsaved changes
    #[serde(skip, default)]
    dirty: bool,
    /// Storage backend for I/O (when set, flush goes through repo)
    #[serde(skip)]
    pub storage: Option<Arc<ClawStorage>>,
    /// Agent ID for storage backend lookups
    #[serde(skip, default)]
    pub agent_id: String,
}

impl CrossSessionMemory {
    // ── Constructors ──

    /// Create memory backed by a storage backend for a specific agent.
    /// Loads existing data from the backend, or returns defaults if none exists.
    pub fn for_agent_with_storage(storage: &Arc<ClawStorage>, agent_id: &str) -> Self {
        let aid = agent_id.to_string();
        let s = storage.clone();
        let mut mem = crate::utils::sync_block_on(async {
            s.memory
                .load(&aid)
                .await
                .unwrap_or_else(|_| None)
                .unwrap_or_else(|| CrossSessionMemory {
                    tool_frequency: HashMap::new(),
                    hot_tools: Vec::new(),
                    preferences: Vec::new(),
                    user_name: None,
                    assistant_nickname: None,
                    user_info: Vec::new(),
                    session_feedback: HashMap::new(),
                    path: PathBuf::new(),
                    dirty: false,
                    storage: None,
                    agent_id: String::new(),
                })
        });
        mem.storage = Some(storage.clone());
        mem.agent_id = aid;
        mem
    }

    /// Legacy: create memory with file path only (backward-compatible).
    #[allow(dead_code)]
    pub fn for_agent(claw_dir: &Path, agent_id: &str) -> Self {
        let path = claw_dir.join("agents").join(agent_id).join("memory.json");
        let mut mem = Self::new_with_path(path);
        mem.agent_id = agent_id.to_string();
        mem
    }

    #[allow(dead_code)]
    fn new_with_path(path: PathBuf) -> Self {
        let mut mem = if path.exists() {
            Self::load(&path)
        } else {
            Self::default_memory()
        };
        mem.path = path;
        mem
    }

    pub(crate) fn default_memory() -> Self {
        Self {
            tool_frequency: HashMap::new(),
            hot_tools: Vec::new(),
            preferences: Vec::new(),
            user_name: None,
            assistant_nickname: None,
            user_info: Vec::new(),
            session_feedback: HashMap::new(),
            path: PathBuf::new(),
            dirty: false,
            storage: None,
            agent_id: String::new(),
        }
    }

    /// Load from a JSON file path (public for FileBackend).
    pub fn load_from(path: &Path) -> Self {
        if let Ok(content) = std::fs::read_to_string(path)
            && let Ok(mut mem) = serde_json::from_str::<Self>(&content)
        {
            mem.path = path.to_path_buf();
            return mem;
        }
        let mut mem = Self::default_memory();
        mem.path = path.to_path_buf();
        mem
    }

    #[allow(dead_code)]
    fn load(path: &Path) -> Self {
        Self::load_from(path)
    }

    // =============================================
    // Tool frequency tracking
    // =============================================

    /// Get tool frequency map (for context compression decisions)
    pub fn tool_frequency(&self) -> &HashMap<String, usize> {
        &self.tool_frequency
    }

    /// Record a tool call to update frequency.
    pub fn record_tool_use(&mut self, tool_name: &str) {
        let count = self
            .tool_frequency
            .entry(tool_name.to_string())
            .or_insert(0);
        *count += 1;
        self.update_hot_tools();
        self.dirty = true;
    }

    /// Update hot_tools list sorted by frequency (descending), top 5.
    fn update_hot_tools(&mut self) {
        let mut tools: Vec<(String, usize)> = self
            .tool_frequency
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        tools.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        self.hot_tools = tools.into_iter().map(|(t, _)| t).take(5).collect();
    }

    /// Analyze tool usage from session JSONL files.
    pub fn analyze_sessions(
        &mut self,
        sessions: &[crate::session::SessionMeta],
        session_mgr: &crate::session::SessionManager,
    ) {
        for meta in sessions {
            let messages = session_mgr.load_app_messages(&meta.id, 1000);
            for msg in &messages {
                if let crate::app::Message::ToolCall { name, .. } = msg {
                    *self.tool_frequency.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }
        self.update_hot_tools();
        self.flush();
    }

    // =============================================
    // User preferences & profile
    // =============================================

    /// Whether user has completed basic onboarding (name or info recorded)
    pub fn has_user_profile(&self) -> bool {
        self.user_name.is_some() || !self.user_info.is_empty()
    }

    /// Get the assistant nickname if set.
    pub fn assistant_nickname(&self) -> Option<&str> {
        self.assistant_nickname.as_deref()
    }

    /// Set user's name
    pub fn set_user_name(&mut self, name: &str) {
        self.user_name = Some(name.to_string());
        self.dirty = true;
    }

    /// Set what the user calls this assistant.
    pub fn set_assistant_nickname(&mut self, name: &str) {
        self.assistant_nickname = Some(name.to_string());
        self.dirty = true;
    }

    /// Add a user info fact (deduplicated).
    pub fn add_user_info(&mut self, info: &str) {
        let p = info.to_string();
        if !self.user_info.contains(&p) {
            self.user_info.push(p);
            self.dirty = true;
        }
    }

    /// Add a user preference (deduplicated).
    pub fn add_preference(&mut self, pref: &str) {
        let p = pref.to_string();
        if !self.preferences.contains(&p) {
            self.preferences.push(p);
            self.dirty = true;
        }
    }

    /// Record session feedback (thumbs up/down).
    #[allow(dead_code)]
    pub fn record_session_feedback(&mut self, session_id: &str, positive: bool) {
        let (pos, neg) = self
            .session_feedback
            .entry(session_id.to_string())
            .or_insert((0, 0));
        if positive {
            *pos += 1;
        } else {
            *neg += 1;
        }
        self.dirty = true;
    }

    // =============================================
    // Format for system prompt layers
    // =============================================

    /// Format Layer 3: hot tools with full teach docs.
    #[allow(dead_code)]
    pub fn format_hot_tools(&self, cache: &crate::tool_cache::ToolDocCache) -> String {
        if self.hot_tools.is_empty() {
            return String::new();
        }
        cache.format_hot_tools(&self.hot_tools)
    }

    /// Format Layer 4: user memory section.
    pub fn format_user_memory(&self) -> String {
        let has_content = !self.hot_tools.is_empty()
            || !self.preferences.is_empty()
            || self.user_name.is_some()
            || self.assistant_nickname.is_some()
            || !self.user_info.is_empty();
        if !has_content {
            return String::new();
        }

        let mut result = String::from("## 用户记忆\n");
        if let Some(ref name) = self.user_name {
            result.push_str(&format!("用户称呼：{}\n", name));
        }
        if let Some(ref nick) = self.assistant_nickname {
            result.push_str(&format!("用户称呼你为：{}\n", nick));
        }
        if !self.hot_tools.is_empty() {
            result.push_str(&format!("常用工具：{}\n", self.hot_tools.join(", ")));
        }
        for pref in &self.preferences {
            result.push_str(&format!("偏好：{}\n", pref));
        }
        for info in &self.user_info {
            result.push_str(&format!("用户信息：{}\n", info));
        }
        result
    }

    /// Format user profile section for system prompt onboarding.
    pub fn format_user_profile(&self) -> String {
        if self.has_user_profile() {
            let mut result = String::from("## 认识用户\n");
            if let Some(ref name) = self.user_name {
                result.push_str(&format!("用户称呼：{}\n", name));
            }
            for info in &self.user_info {
                result.push_str(&format!("用户信息：{}\n", info));
            }
            result
        } else {
            String::from(
                "## 认识用户\n\
                 这是一个新用户，你还没有充分了解对方。\n\
                 请主动询问用户的称呼、兴趣爱好和生活习惯，\n\
                 在对话中自然地了解并记住这些信息。\n\
                 例如：「请问我怎么称呼您？」「您平时有什么兴趣爱好？」\n",
            )
        }
    }

    // =============================================
    // Disk persistence
    // =============================================

    /// Flush pending changes to the storage backend (or file fallback).
    pub fn flush(&mut self) {
        if !self.dirty {
            return;
        }
        if let Some(ref storage) = self.storage {
            let aid = self.agent_id.clone();
            let mem_value = serde_json::to_value(&*self).unwrap_or(serde_json::Value::Null);
            if let Err(e) = crate::utils::sync_block_on(async move {
                let mem: Self =
                    serde_json::from_value(mem_value).unwrap_or_else(|_| Self::default_memory());
                storage.memory.save(&aid, &mem).await
            }) {
                tracing::error!("持久化写入失败: {}", e);
            }
        } else if !self.path.as_os_str().is_empty()
            && let Ok(content) = serde_json::to_string_pretty(self)
            && let Err(e) = atomic_write(&self.path, &content)
        {
            tracing::error!("持久化写入失败: {}", e);
        }
        self.dirty = false;
    }

    /// Async version of [`flush`].
    pub async fn flush_async(&mut self) {
        if !self.dirty {
            return;
        }
        if let Some(ref storage) = self.storage {
            let aid = self.agent_id.clone();
            let mem_value = serde_json::to_value(&*self).unwrap_or(serde_json::Value::Null);
            let mem: Self =
                serde_json::from_value(mem_value).unwrap_or_else(|_| Self::default_memory());
            if let Err(e) = storage.memory.save(&aid, &mem).await {
                tracing::error!("持久化写入失败: {}", e);
            }
        } else if !self.path.as_os_str().is_empty()
            && let Ok(content) = serde_json::to_string_pretty(self)
            && let Err(e) = atomic_write(&self.path, &content)
        {
            tracing::error!("持久化写入失败: {}", e);
        }
        self.dirty = false;
    }

    // ── Access for testing ──

    #[cfg(test)]
    pub(crate) fn test_user_name(&self) -> Option<&str> {
        self.user_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_memory() -> CrossSessionMemory {
        let mut mem = CrossSessionMemory::default_memory();
        mem.path = std::env::temp_dir().join("i-rs-claw-test-memory.json");
        mem
    }

    #[test]
    fn test_record_tool_use() {
        let mut mem = test_memory();
        mem.record_tool_use("weight");
        mem.record_tool_use("weight");
        mem.record_tool_use("mood");

        assert_eq!(mem.tool_frequency.get("weight"), Some(&2));
        assert_eq!(mem.tool_frequency.get("mood"), Some(&1));
        assert_eq!(mem.hot_tools.first().unwrap(), "weight");
        let _ = std::fs::remove_file(&mem.path);
    }

    #[test]
    fn test_has_user_profile() {
        let mut mem = test_memory();
        assert!(!mem.has_user_profile());

        mem.set_user_name("Alice");
        assert!(mem.has_user_profile());
        let _ = std::fs::remove_file(&mem.path);

        let mut mem2 = test_memory();
        mem2.add_user_info("likes coffee");
        assert!(mem2.has_user_profile());
        let _ = std::fs::remove_file(&mem2.path);
    }

    #[test]
    fn test_add_user_info_dedup() {
        let mut mem = test_memory();
        mem.add_user_info("likes coffee");
        mem.add_user_info("likes coffee");
        assert_eq!(mem.user_info.len(), 1);
        let _ = std::fs::remove_file(&mem.path);
    }

    #[test]
    fn test_format_user_memory_empty() {
        let mem = test_memory();
        assert!(mem.format_user_memory().is_empty());
    }

    #[test]
    fn test_format_user_memory_with_data() {
        let mut mem = test_memory();
        mem.set_user_name("Bob");
        mem.add_preference("likes dark mode");
        let output = mem.format_user_memory();
        assert!(output.contains("用户称呼：Bob"));
        assert!(output.contains("likes dark mode"));
        let _ = std::fs::remove_file(&mem.path);
    }

    #[test]
    fn test_format_user_profile_known() {
        let mut mem = test_memory();
        mem.set_user_name("Charlie");
        mem.add_user_info("works from home");
        let output = mem.format_user_profile();
        assert!(output.contains("用户称呼：Charlie"));
        assert!(output.contains("works from home"));
        assert!(!output.contains("新用户"));
        let _ = std::fs::remove_file(&mem.path);
    }

    #[test]
    fn test_format_user_profile_new() {
        let mem = test_memory();
        let output = mem.format_user_profile();
        assert!(output.contains("新用户"));
    }

    // ── Storage-backed memory tests ──

    #[tokio::test]
    async fn test_memory_with_repo_save_load() {
        let id = uuid::Uuid::new_v4().to_string();
        let dir = std::env::temp_dir().join(format!("mem-repo-test-{}", id));
        let _ = std::fs::create_dir_all(&dir);
        let storage = Arc::new(ClawStorage::file(dir.clone()));

        // Load via repo directly (async, no nested block_on)
        let mut mem = storage
            .memory
            .load("agent-a")
            .await
            .unwrap_or_else(|_| None)
            .unwrap_or_else(CrossSessionMemory::default_memory);
        assert!(!mem.has_user_profile());

        mem.set_user_name("TestUser");
        mem.add_preference("dark theme");
        storage.memory.save("agent-a", &mem).await.unwrap();

        // Load again to verify persistence
        let loaded = storage
            .memory
            .load("agent-a")
            .await
            .unwrap_or_else(|e| panic!("memory load failed: {}", e))
            .unwrap_or_else(|| panic!("memory for 'agent-a' missing after save"));
        assert!(loaded.has_user_profile());
        assert_eq!(loaded.test_user_name(), Some("TestUser"));
        assert!(loaded.preferences.contains(&"dark theme".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
