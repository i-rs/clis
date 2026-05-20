use crate::utils::atomic_write;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cached tool documentation for multi-layer system prompt assembly.
///
/// Layer 2 is built from static TOOL_INDEX data (tool name + description).
/// Layer 3 is fetched at session start via `i-rs <tool> skill teach` for
/// frequently used tools, then cached to disk for reuse.
#[derive(Debug, Clone)]
pub struct ToolDocCache {
    /// Cached `skill teach` output keyed by tool name
    pub hot_docs: HashMap<String, String>,
    /// Cache directory (~/.config/i-rs/claw/)
    #[allow(dead_code)]
    cache_dir: PathBuf,
}

#[allow(dead_code)]
impl ToolDocCache {
    /// Create tool cache for a specific agent.
    /// "default" uses legacy global cache dir; others use
    /// `claw_dir/agents/{agent_id}/`.
    pub fn for_agent(cache_dir: &Path, agent_id: &str) -> Self {
        let dir = if agent_id == "default" {
            cache_dir.to_path_buf()
        } else {
            cache_dir.join("agents").join(agent_id)
        };
        Self::new(dir)
    }

    pub fn new(cache_dir: PathBuf) -> Self {
        let hot_docs = Self::load_hot_docs(&cache_dir);

        Self {
            hot_docs,
            cache_dir,
        }
    }

    /// Build Layer 3 section: full skill teach docs for specified tools.
    pub fn format_hot_tools(&self, tools: &[String]) -> String {
        if tools.is_empty() {
            return String::new();
        }

        let mut result = String::from("## 常用工具文档\n\n以下是你最近常用的工具的完整教学文档：\n");

        for tool in tools {
            if let Some(doc) = self.hot_docs.get(tool) {
                result.push_str(&format!("\n### {}\n{}", tool, doc));
            }
        }

        if result == "## 常用工具文档\n\n以下是你最近常用的工具的完整教学文档：\n" {
            return String::new();
        }

        result
    }

    /// Format Layer 4: user preferences and cross-session info.
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
        for tool in tools {
            if !self.hot_docs.contains_key(tool)
                && let Some(doc) = Self::fetch_teach_doc(tool) {
                    self.hot_docs.insert(tool.clone(), doc);
                }
        }
        self.save_hot_docs();
    }

    // --- Disk cache ---

    fn cache_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("hot_docs_cache.json")
    }

    fn load_hot_docs(cache_dir: &Path) -> HashMap<String, String> {
        let path = Self::cache_path(cache_dir);
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(map) = serde_json::from_str(&content) {
                    return map;
                }
        HashMap::new()
    }

    fn save_hot_docs(&self) {
        let path = Self::cache_path(&self.cache_dir);
        if let Ok(content) = serde_json::to_string(&self.hot_docs) {
            let _ = atomic_write(&path, &content);
        }
    }
}
