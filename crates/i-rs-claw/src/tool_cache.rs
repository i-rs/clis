use std::collections::HashMap;
use std::path::PathBuf;

/// Cached tool documentation for multi-layer system prompt assembly.
///
/// Layer 2 is built from static TOOL_INDEX data (tool name + description).
/// Layer 3 is fetched at session start via `i-rs <tool> skill teach` for
/// frequently used tools, then cached to disk for reuse.
pub struct ToolDocCache {
    /// Layer 2: compact tool index text
    pub index_text: String,
    /// Layer 3: cached `skill teach` output keyed by tool name
    pub hot_docs: HashMap<String, String>,
    /// Cache directory (~/.config/i-rs/claw/)
    #[allow(dead_code)]
    cache_dir: PathBuf,
}

#[allow(dead_code)]
impl ToolDocCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        let index_text = crate::tools::search::format_index();
        let hot_docs = Self::load_hot_docs(&cache_dir);

        Self {
            index_text,
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
            if !self.hot_docs.contains_key(tool) {
                if let Some(doc) = Self::fetch_teach_doc(tool) {
                    self.hot_docs.insert(tool.clone(), doc);
                }
            }
        }
        self.save_hot_docs();
    }

    // --- Disk cache ---

    fn cache_path(cache_dir: &PathBuf) -> PathBuf {
        cache_dir.join("hot_docs_cache.json")
    }

    fn load_hot_docs(cache_dir: &PathBuf) -> HashMap<String, String> {
        let path = Self::cache_path(cache_dir);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(map) = serde_json::from_str(&content) {
                    return map;
                }
            }
        }
        HashMap::new()
    }

    fn save_hot_docs(&self) {
        let path = Self::cache_path(&self.cache_dir);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string(&self.hot_docs) {
            let _ = std::fs::write(&path, content);
        }
    }
}
