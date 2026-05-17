use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Cross-session memory that tracks tool usage patterns and user preferences.
///
/// Persisted to disk as a JSON file and updated after each conversation turn.
/// Feeds into the multi-layer system prompt (Layer 3: hot tools, Layer 4: user memory).
#[derive(Debug, Serialize, Deserialize)]
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
    /// Free-form user info facts (habits, preferences, etc.)
    #[serde(default)]
    user_info: Vec<String>,
    /// Path to disk cache file
    #[serde(skip)]
    path: PathBuf,
}

impl CrossSessionMemory {
    pub fn new(claw_dir: PathBuf) -> Self {
        let path = claw_dir.join("memory.json");
        let mut mem = if path.exists() {
            Self::load(&path)
        } else {
            Self {
                tool_frequency: HashMap::new(),
                hot_tools: Vec::new(),
                preferences: Vec::new(),
                user_name: None,
                user_info: Vec::new(),
                path: path.clone(),
            }
        };
        mem.path = path;
        mem
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
        let count = self.tool_frequency.entry(tool_name.to_string()).or_insert(0);
        *count += 1;
        self.update_hot_tools();
        self.save();
    }

    /// Update hot_tools list sorted by frequency (descending), top 5.
    fn update_hot_tools(&mut self) {
        let mut tools: Vec<(String, usize)> = self
            .tool_frequency
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        tools.sort_by(|a, b| b.1.cmp(&a.1));
        self.hot_tools = tools.into_iter().map(|(t, _)| t).take(5).collect();
    }

    /// Analyze tool usage from session JSONL files.
    pub fn analyze_sessions(
        &mut self,
        sessions: &[crate::session::SessionMeta],
        session_mgr: &crate::session::SessionManager,
    ) {
        for meta in sessions {
            let records = session_mgr.load_messages(&meta.id, 1000);
            for record in &records {
                if record.get("type").and_then(|t| t.as_str()) == Some("tool_call") {
                    if let Some(name) = record.get("name").and_then(|n| n.as_str()) {
                        *self.tool_frequency.entry(name.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        self.update_hot_tools();
        self.save();
    }

    // =============================================
    // User preferences & profile
    // =============================================

    /// Whether user has completed basic onboarding (name or info recorded)
    pub fn has_user_profile(&self) -> bool {
        self.user_name.is_some() || !self.user_info.is_empty()
    }

    /// Get user's name
    #[allow(dead_code)]
    pub fn user_name(&self) -> Option<&str> {
        self.user_name.as_deref()
    }

    /// Set user's name
    pub fn set_user_name(&mut self, name: &str) {
        self.user_name = Some(name.to_string());
        self.save();
    }

    /// Add a user info fact (deduplicated).
    pub fn add_user_info(&mut self, info: &str) {
        let p = info.to_string();
        if !self.user_info.contains(&p) {
            self.user_info.push(p);
            self.save();
        }
    }

    /// Add a user preference (deduplicated).
    pub fn add_preference(&mut self, pref: &str) {
        let p = pref.to_string();
        if !self.preferences.contains(&p) {
            self.preferences.push(p);
            self.save();
        }
    }

    // =============================================
    // Format for system prompt layers
    // =============================================

    /// Format Layer 3: hot tools with full teach docs.
    pub fn format_hot_tools(&self, cache: &crate::tool_cache::ToolDocCache) -> String {
        if self.hot_tools.is_empty() {
            return String::new();
        }
        cache.format_hot_tools(&self.hot_tools)
    }

    /// Format Layer 4: user memory section.
    pub fn format_user_memory(&self) -> String {
        if self.hot_tools.is_empty()
            && self.preferences.is_empty()
            && self.user_name.is_none()
            && self.user_info.is_empty()
        {
            return String::new();
        }

        let mut result = String::from("## 用户记忆\n");
        if let Some(ref name) = self.user_name {
            result.push_str(&format!("用户称呼：{}\n", name));
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
    /// Returns a prompt snippet that guides the AI on user familiarity.
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

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&self) {
            let _ = std::fs::write(&self.path, content);
        }
    }

    fn load(path: &PathBuf) -> Self {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(mut mem) = serde_json::from_str::<Self>(&content) {
                mem.path = path.clone();
                return mem;
            }
        }
        Self {
            tool_frequency: HashMap::new(),
            hot_tools: Vec::new(),
            preferences: Vec::new(),
            user_name: None,
            user_info: Vec::new(),
            path: path.clone(),
        }
    }
}
