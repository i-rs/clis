use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowelEntry {
    pub id: String,
    pub towel_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub replaced_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl TowelEntry {
    pub fn new(towel_type: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            towel_type,
            replaced_at: now,
            tags,
            remark,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TowelStore {
    pub entries: BTreeMap<String, TowelEntry>,
}

impl TowelStore {
    pub fn add_entry(&mut self, entry: TowelEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<TowelEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&TowelEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct TowelRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "TYPE")]
    towel_type: String,
    #[tabled(rename = "REPLACED_AT")]
    replaced_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl TowelRow {
    pub fn from_entry(entry: &TowelEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            towel_type: entry.towel_type.clone(),
            replaced_at: entry.replaced_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: if entry.tags.is_empty() {
                "-".to_string()
            } else {
                entry.tags.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub towel_type: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub replaced_at: String,
}

impl From<&TowelEntry> for ListItem {
    fn from(entry: &TowelEntry) -> Self {
        Self {
            id: entry.id.clone(),
            towel_type: entry.towel_type.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            replaced_at: entry.replaced_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
