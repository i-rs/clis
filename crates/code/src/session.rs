use crate::app::AgentMessage;
use crate::provider::LlmMessage;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub messages: Vec<AgentMessage>,
    #[serde(default)]
    pub agent_messages: Vec<LlmMessage>,
    pub created_at: String,
    pub updated_at: String,
}

impl Session {
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: Vec::new(),
            agent_messages: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn from_agent_messages(id: Option<String>, msgs: &[AgentMessage], agent_msgs: Vec<LlmMessage>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            messages: msgs.to_vec(),
            agent_messages: agent_msgs,
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
    use crate::app::AgentMessage;

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
        s.messages.push(AgentMessage::user("hello"));
        s.messages.push(AgentMessage::assistant("hi there"));
        s.save(&dir).expect("save should work");
        let loaded = Session::load(&s.id, &dir).expect("load should work");
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(
            serde_json::to_string(&loaded.messages[0]).unwrap(),
            serde_json::to_string(&s.messages[0]).unwrap()
        );
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
        use serde_json::json;
        let dir = std::env::temp_dir().join("i-rs-code-test-session-tc");
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Session::new();
        let tc = json!({"id": "call_1", "name": "bash", "args": {"command": "ls"}});
        s.messages.push(AgentMessage::Assistant {
            content: String::new(),
            reasoning: String::new(),
            tool_calls: Some(vec![tc]),
            reasoning_expanded: false,
        });
        s.save(&dir).expect("save with tool_calls should work");
        let loaded = Session::load(&s.id, &dir).expect("load with tool_calls should work");
        match &loaded.messages[0] {
            AgentMessage::Assistant { tool_calls, .. } => {
                assert!(tool_calls.is_some());
            }
            _ => panic!("expected Assistant"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_agent_messages_roundtrip() {
        let dir = std::env::temp_dir().join("i-rs-code-test-session-agent-msgs");
        let _ = std::fs::remove_dir_all(&dir);
        let s = Session::from_agent_messages(
            Some("test-id".into()),
            &[AgentMessage::user("hello")],
            vec![
                crate::provider::LlmMessage::System("you are helpful".into()),
                crate::provider::LlmMessage::User("hello".into()),
            ],
        );
        s.save(&dir).expect("save should work");
        let loaded = Session::load("test-id", &dir).expect("load should work");
        assert_eq!(loaded.agent_messages.len(), 2);
        assert!(matches!(&loaded.agent_messages[0], crate::provider::LlmMessage::System(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
