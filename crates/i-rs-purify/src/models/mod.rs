use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurifyEntry {
    pub id: String,
    pub filter_type: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub replaced_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl PurifyEntry {
    pub fn new(filter_type: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, filter_type, replaced_at: now, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurifyStore {
    pub entries: BTreeMap<String, PurifyEntry>,
}

impl Default for PurifyStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl PurifyStore {
    pub fn add_entry(&mut self, entry: PurifyEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<PurifyEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&PurifyEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct PurifyRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "FILTER_TYPE")]
    filter_type: String,
    #[tabled(rename = "REPLACED_AT")]
    replaced_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl PurifyRow {
    pub fn from_entry(entry: &PurifyEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            filter_type: entry.filter_type.clone(),
            replaced_at: entry.replaced_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: if entry.tags.is_empty() { "-".to_string() } else { entry.tags.join(", ") },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub filter_type: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub replaced_at: String,
}

impl From<&PurifyEntry> for ListItem {
    fn from(entry: &PurifyEntry) -> Self {
        Self {
            id: entry.id.clone(),
            filter_type: entry.filter_type.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            replaced_at: entry.replaced_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
