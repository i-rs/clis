use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub session_id: String,
    pub message_type: String,
    pub excerpt: String,
}

pub struct ConvStore {
    sessions_dir: PathBuf,
}

impl ConvStore {
    pub fn new(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        if !self.sessions_dir.exists() {
            return results;
        }
        let entries = match std::fs::read_dir(&self.sessions_dir) {
            Ok(e) => e,
            Err(_) => return results,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let session: Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            if let Some(messages) = session.get("messages").and_then(|m| m.as_array()) {
                for msg in messages {
                    let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
                    let text = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    if text.to_lowercase().contains(&query_lower) {
                        results.push(SearchResult {
                            session_id: session_id.clone(),
                            message_type: role.to_string(),
                            excerpt: text.chars().take(200).collect(),
                        });
                        if results.len() >= max_results {
                            return results;
                        }
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_empty_dir() {
        let dir = std::env::temp_dir().join("i-rs-code-test-convstore-empty");
        let _ = std::fs::remove_dir_all(&dir);
        let store = ConvStore::new(dir);
        assert!(store.search("hello", 10).is_empty());
    }

    #[test]
    fn test_search_finds_match() {
        let dir = std::env::temp_dir().join("i-rs-code-test-convstore-find");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let data = serde_json::json!({
            "messages": [
                {"role": "user", "content": "create a file"},
                {"role": "assistant", "content": "I'll create that file"}
            ]
        });
        std::fs::write(
            dir.join("test-session.json"),
            serde_json::to_string(&data).unwrap(),
        )
        .unwrap();
        let store = ConvStore::new(dir.clone());
        let results = store.search("create", 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].message_type, "user");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
