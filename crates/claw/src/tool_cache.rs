use i_rs_claw_core::storage::ClawStorage;
use i_rs_claw_core::utils::atomic_write;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cached tool documentation for multi-layer system prompt assembly.
///
/// Layer 2 is built from static TOOL_INDEX data (tool name + description).
/// Layer 3 is fetched at session start via `i-rs <tool> skill teach` for
/// frequently used tools, then cached to the storage backend for reuse.
#[derive(Debug, Clone)]
pub struct ToolDocCache {
    /// Cached `skill teach` output keyed by tool name
    pub hot_docs: HashMap<String, String>,
    /// Cache directory (file fallback)
    cache_dir: PathBuf,
    /// Storage backend (takes priority over file when set)
    storage: Option<Arc<ClawStorage>>,
    agent_id: String,
}

impl ToolDocCache {
    /// Create tool cache backed by a storage backend.
    pub fn for_agent_with_storage(storage: &Arc<ClawStorage>, agent_id: &str) -> Self {
        let aid = agent_id.to_string();
        let s = storage.clone();
        let hot_docs = i_rs_claw_core::utils::sync_block_on(async {
            s.tool_cache.load(&aid).await.unwrap_or_default()
        });
        Self {
            hot_docs,
            cache_dir: PathBuf::new(),
            storage: Some(storage.clone()),
            agent_id: aid,
        }
    }

    /// Legacy: create with file backend.
    #[allow(dead_code)]
    pub fn for_agent(cache_dir: &Path, agent_id: &str) -> Self {
        let dir = cache_dir.join("agents").join(agent_id);
        Self::new_with_dir(dir, agent_id)
    }

    #[allow(dead_code)]
    pub fn new(cache_dir: PathBuf) -> Self {
        Self::new_with_dir(cache_dir, "default")
    }

    #[allow(dead_code)]
    fn new_with_dir(cache_dir: PathBuf, agent_id: &str) -> Self {
        let hot_docs = Self::load_hot_docs(&cache_dir);
        Self {
            hot_docs,
            cache_dir,
            storage: None,
            agent_id: agent_id.to_string(),
        }
    }

    /// Build Layer 3 section: full teach docs for hot tools.
    /// These stay in the system prompt so the LLM does not need to
    /// re-query `skill teach` after compression drops earlier results.
    pub fn format_hot_tools(&self, tools: &[String]) -> String {
        if tools.is_empty() {
            return String::new();
        }

        let mut result = String::from(
            "## 常用工具文档\n\n以下是你最近常用的工具的完整教学文档（只需开始一次，无需重复 teach）：\n",
        );

        let mut any = false;
        for tool in tools.iter().take(5) {
            if let Some(doc) = self.hot_docs.get(tool) {
                result.push_str(&format!("\n### {}\n{}", tool, doc));
                any = true;
            }
        }

        if !any {
            return String::new();
        }

        result
    }

    /// Format Layer 4: user preferences and cross-session info.
    #[allow(dead_code)]
    pub fn format_user_memory(preferences: &[String], hot_tools: &[String]) -> String {
        if preferences.is_empty() && hot_tools.is_empty() {
            return String::new();
        }

        let mut result = String::from("## 用户记忆\n");
        if !hot_tools.is_empty() {
            result.push_str(&format!("常用工具：{}\n", hot_tools.join(", ")));
        }
        for pref in preferences {
            result.push_str(&format!("偏好：{}\n", pref));
        }
        result
    }

    /// Fetch `skill teach` output for a tool via CLI.
    pub fn fetch_teach_doc(tool: &str) -> Option<String> {
        let output = std::process::Command::new("i-rs")
            .arg(tool)
            .arg("skill")
            .arg("teach")
            .output()
            .ok()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        } else {
            None
        }
    }

    /// Prefetch teaching docs for a set of tools and cache them.
    pub fn prefetch(&mut self, tools: &[String]) {
        let mut changed = false;
        for tool in tools {
            if let Some(doc) = Self::fetch_teach_doc(tool)
                && self.hot_docs.get(tool) != Some(&doc)
            {
                self.hot_docs.insert(tool.to_string(), doc);
                changed = true;
            }
        }
        if changed {
            self.save_hot_docs();
        }
    }

    /// Save hot docs to the storage backend (or file fallback).
    pub fn save_hot_docs(&self) {
        if let Some(ref storage) = self.storage {
            let aid = self.agent_id.clone();
            let docs = self.hot_docs.clone();
            if let Err(e) =
                i_rs_claw_core::utils::sync_block_on(
                    async move { storage.tool_cache.save(&aid, &docs).await },
                )
            {
                tracing::error!("持久化写入失败: {}", e);
            }
            return;
        }
        // File fallback
        let path = Self::cache_path(&self.cache_dir);
        if let Ok(content) = serde_json::to_string(&self.hot_docs)
            && let Err(e) = atomic_write(&path, &content)
        {
            tracing::error!("持久化写入失败: {}", e);
        }
    }

    // --- File fallback ---

    fn cache_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("hot_docs_cache.json")
    }

    #[allow(dead_code)]
    fn load_hot_docs(cache_dir: &Path) -> HashMap<String, String> {
        let path = Self::cache_path(cache_dir);
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(map) = serde_json::from_str(&content)
        {
            return map;
        }
        HashMap::new()
    }
}
