use std::collections::HashMap;
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(30);

pub struct ToolResultCache {
    entries: HashMap<String, CachedResult>,
}

struct CachedResult {
    value: String,
    inserted_at: Instant,
}

impl ToolResultCache {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    fn key(tool_name: &str, args_json: &str) -> String {
        format!("{}::{}", tool_name, args_json)
    }

    pub fn get(&self, tool_name: &str, args_json: &str) -> Option<&str> {
        let key = Self::key(tool_name, args_json);
        self.entries.get(&key).and_then(|c| {
            if c.inserted_at.elapsed() > CACHE_TTL {
                None
            } else {
                Some(c.value.as_str())
            }
        })
    }

    pub fn insert(&mut self, tool_name: &str, args_json: &str, value: String) {
        let key = Self::key(tool_name, args_json);
        self.entries.insert(key, CachedResult {
            value,
            inserted_at: Instant::now(),
        });
    }

    pub fn invalidate_all(&mut self) {
        self.entries.clear();
    }

    pub fn is_cacheable(tool_name: &str) -> bool {
        matches!(tool_name, "read" | "glob" | "grep" | "ls")
    }

    pub fn is_mutator(tool_name: &str) -> bool {
        matches!(tool_name, "write" | "edit" | "delete" | "rename" | "batch_edit")
    }
}
