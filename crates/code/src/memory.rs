use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSessionMemory {
    #[serde(default)]
    tool_frequency: HashMap<String, usize>,
    #[serde(default)]
    tool_last_used: HashMap<String, u64>,
    #[serde(default)]
    preferences: Vec<String>,
    #[serde(default)]
    project_context: Vec<String>,
    #[serde(default)]
    project_hash: String,
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip, default)]
    dirty: bool,
}

impl CrossSessionMemory {
    pub fn new(dir: &Path, project_hash: &str) -> Self {
        let filename = if project_hash.is_empty() {
            "memory.json".to_string()
        } else {
            format!("memory-{}.json", project_hash)
        };
        let path = dir.join(&filename);
        let mut mem = if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Self::empty())
        } else {
            Self::empty()
        };
        mem.path = path;
        mem.project_hash = project_hash.to_string();
        mem
    }

    fn empty() -> Self {
        Self {
            tool_frequency: HashMap::new(),
            tool_last_used: HashMap::new(),
            preferences: Vec::new(),
            project_context: Vec::new(),
            project_hash: String::new(),
            path: PathBuf::new(),
            dirty: false,
        }
    }

    pub fn record_tool_use(&mut self, name: &str) {
        *self.tool_frequency.entry(name.to_string()).or_insert(0) += 1;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        self.tool_last_used.insert(name.to_string(), now);
        self.dirty = true;
    }

    pub fn add_preference(&mut self, pref: &str) {
        if !self.preferences.iter().any(|p| p == pref) {
            self.preferences.push(pref.to_string());
            self.dirty = true;
        }
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        if !self.dirty { return Ok(()); }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&self.path, content)?;
        self.dirty = false;
        Ok(())
    }

    pub fn format_for_prompt(&self) -> String {
        let mut parts = Vec::new();
        if !self.preferences.is_empty() {
            parts.push(format!("User preferences: {}", self.preferences.join("; ")));
        }
        if !self.project_context.is_empty() {
            parts.push(format!("Project context: {}", self.project_context.join("; ")));
        }
        if !self.tool_frequency.is_empty() {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            let day = 86400u64;
            let mut scored: Vec<(&String, f64)> = self.tool_frequency.iter()
                .map(|(name, count)| {
                    let last = self.tool_last_used.get(name).copied().unwrap_or(0);
                    let days_since = now.saturating_sub(last) / day;
                    let recency = 1.0 / (1.0 + days_since as f64);
                    let score = *count as f64 * recency;
                    (name, score)
                })
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let top: Vec<&str> = scored.iter().take(5).map(|(n, _)| n.as_str()).collect();
            parts.push(format!("Frequently used tools: {}", top.join(", ")));
        }
        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_format() {
        let dir = std::env::temp_dir().join("i-rs-code-test-memory");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut mem = CrossSessionMemory::new(&dir, "");
        mem.record_tool_use("read");
        mem.record_tool_use("bash");
        mem.record_tool_use("read");
        mem.add_preference("use rustfmt");
        let prompt = mem.format_for_prompt();
        assert!(prompt.contains("read"));
        assert!(prompt.contains("use rustfmt"));
        mem.flush().unwrap();
        let loaded = CrossSessionMemory::new(&dir, "");
        assert_eq!(loaded.tool_frequency.get("read"), Some(&2));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_memory() {
        let dir = std::env::temp_dir().join("i-rs-code-test-memory-empty");
        let _ = std::fs::remove_dir_all(&dir);
        let mem = CrossSessionMemory::new(&dir, "");
        assert!(mem.format_for_prompt().is_empty());
    }

    #[test]
    fn test_no_duplicate_preferences() {
        let dir = std::env::temp_dir().join("i-rs-code-test-mem-dedup");
        let _ = std::fs::remove_dir_all(&dir);
        let mut mem = CrossSessionMemory::new(&dir, "");
        mem.add_preference("use tabs");
        mem.add_preference("use tabs");
        assert_eq!(mem.preferences.len(), 1);
    }

    #[test]
    fn test_project_isolation() {
        let dir = std::env::temp_dir().join("i-rs-code-test-mem-project");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut mem_a = CrossSessionMemory::new(&dir, "proj_a");
        mem_a.record_tool_use("read");
        mem_a.flush().unwrap();
        let mem_b = CrossSessionMemory::new(&dir, "proj_b");
        assert!(mem_b.tool_frequency.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_recency_scoring() {
        let dir = std::env::temp_dir().join("i-rs-code-test-recency");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut mem = CrossSessionMemory::new(&dir, "");
        mem.record_tool_use("old_tool");
        if let Some(last) = mem.tool_last_used.get_mut("old_tool") {
            *last = 100_000; // ~1 day after epoch, very old
        }
        mem.record_tool_use("new_tool");
        let prompt = mem.format_for_prompt();
        assert!(prompt.contains("new_tool"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
