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
                tool_calls: None,
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
