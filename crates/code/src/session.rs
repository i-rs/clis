use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub reasoning: String,
    pub tool_calls: Option<Vec<Value>>,
}

impl Session {
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn from_chat_messages(id: Option<String>, msgs: &[crate::app::ChatMessage]) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            messages: msgs.iter().map(|m| Message {
                role: m.role.clone(),
                content: m.content.clone(),
                reasoning: m.reasoning.clone(),
                tool_calls: m.tool_calls.clone(),
            }).collect(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn save(&self, sessions_dir: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(sessions_dir)?;
        let path = sessions_dir.join(format!("{}.json", self.id));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn load(id: &str, sessions_dir: &Path) -> anyhow::Result<Self> {
        let path = sessions_dir.join(format!("{}.json", id));
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn list(sessions_dir: &Path) -> anyhow::Result<Vec<String>> {
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for entry in std::fs::read_dir(sessions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && let Some(name) = entry.file_name().to_str()
                && let Some(id) = name.strip_suffix(".json")
            {
                ids.push(id.to_string());
            }
        }
        ids.sort();
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let s = Session::new();
        assert!(!s.id.is_empty());
        assert!(s.messages.is_empty());
        assert!(!s.created_at.is_empty());
    }

    #[test]
    fn test_session_roundtrip() {
        let dir = std::env::temp_dir().join("i-rs-code-test-session-roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Session::new();
        s.messages.push(Message { role: "user".into(), content: "hello".into(), reasoning: String::new(), tool_calls: None });
        s.messages.push(Message { role: "assistant".into(), content: "hi there".into(), reasoning: String::new(), tool_calls: None });
        s.save(&dir).expect("save should work");
        let loaded = Session::load(&s.id, &dir).expect("load should work");
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[0].content, "hello");
        assert_eq!(loaded.messages[1].content, "hi there");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_list() {
        let dir = std::env::temp_dir().join("i-rs-code-test-session-list");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("aaa.json"), "{}").unwrap();
        std::fs::write(dir.join("bbb.json"), "{}").unwrap();
        std::fs::write(dir.join("readme.txt"), "").unwrap();
        let ids = Session::list(&dir).unwrap();
        assert_eq!(ids, vec!["aaa", "bbb"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_tool_calls_persist() {
        let dir = std::env::temp_dir().join("i-rs-code-test-session-tc");
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Session::new();
        let tc = serde_json::json!({"id": "call_1", "name": "bash", "args": {"command": "ls"}});
        s.messages.push(Message { role: "assistant".into(), content: String::new(), reasoning: String::new(), tool_calls: Some(vec![tc]) });
        s.save(&dir).expect("save with tool_calls should work");
        let loaded = Session::load(&s.id, &dir).expect("load with tool_calls should work");
        assert!(loaded.messages[0].tool_calls.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
