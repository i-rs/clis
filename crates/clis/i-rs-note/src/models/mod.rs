use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteStore {
    pub notes: std::collections::BTreeMap<String, Note>,
}

#[derive(Tabled)]
pub struct NoteRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TITLE")]
    title: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl NoteStore {
    pub fn add_entry(&mut self, entry: Note) {
        self.notes.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<Note> {
        self.notes.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&Note> {
        self.notes.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut Note> {
        self.notes.get_mut(key)
    }
}

impl NoteRow {
    pub fn from_note(note: &Note) -> Self {
        Self {
            name: note.name.clone(),
            title: note.title.clone().unwrap_or_else(|| "-".to_string()),
            tags: if note.tags.is_empty() {
                "-".to_string()
            } else {
                note.tags.join(", ")
            },
            created_at: note.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: note.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
