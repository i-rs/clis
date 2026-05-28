use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSessionMemory {
    #[serde(default)]
    tool_frequency: HashMap<String, usize>,
    #[serde(default)]
    preferences: Vec<String>,
    #[serde(default)]
    project_context: Vec<String>,
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip, default)]
    dirty: bool,
}

impl CrossSessionMemory {
    pub fn new(dir: &Path) -> Self {
        let path = dir.join("memory.json");
        let mut mem = if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Self::empty())
        } else {
            Self::empty()
        };
        mem.path = path;
        mem
    }

    fn empty() -> Self {
        Self {
            tool_frequency: HashMap::new(),
            preferences: Vec::new(),
            project_context: Vec::new(),
            path: PathBuf::new(),
            dirty: false,
        }
    }

    pub fn record_tool_use(&mut self, name: &str) {
        *self.tool_frequency.entry(name.to_string()).or_insert(0) += 1;
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
            let top: Vec<_> = self.tool_frequency.iter()
                .collect::<Vec<_>>();
            let mut top = top;
            top.sort_by(|a, b| b.1.cmp(a.1));
            let names: Vec<_> = top.iter().take(5).map(|(n, _)| n.as_str()).collect();
            parts.push(format!("Frequently used tools: {}", names.join(", ")));
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
        let mut mem = CrossSessionMemory::new(&dir);
        mem.record_tool_use("read");
        mem.record_tool_use("bash");
        mem.record_tool_use("read");
        mem.add_preference("use rustfmt");
        let prompt = mem.format_for_prompt();
        assert!(prompt.contains("read"));
        assert!(prompt.contains("use rustfmt"));
        mem.flush().unwrap();
        let loaded = CrossSessionMemory::new(&dir);
        assert_eq!(loaded.tool_frequency.get("read"), Some(&2));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty_memory() {
        let dir = std::env::temp_dir().join("i-rs-code-test-memory-empty");
        let _ = std::fs::remove_dir_all(&dir);
        let mem = CrossSessionMemory::new(&dir);
        assert!(mem.format_for_prompt().is_empty());
    }

    #[test]
    fn test_no_duplicate_preferences() {
        let dir = std::env::temp_dir().join("i-rs-code-test-mem-dedup");
        let _ = std::fs::remove_dir_all(&dir);
        let mut mem = CrossSessionMemory::new(&dir);
        mem.add_preference("use tabs");
        mem.add_preference("use tabs");
        assert_eq!(mem.preferences.len(), 1);
    }
}
