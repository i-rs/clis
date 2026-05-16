use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquaEntry {
    pub id: String,
    pub tank_size_liters: Option<i32>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub changed_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl AquaEntry {
    pub fn new(tank_size_liters: Option<i32>, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            tank_size_liters,
            changed_at: now,
            tags,
            remark,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AquaStore {
    pub entries: BTreeMap<String, AquaEntry>,
}

impl AquaStore {
    pub fn add_entry(&mut self, entry: AquaEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<AquaEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&AquaEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct AquaRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "SIZE")]
    tank_size: String,
    #[tabled(rename = "CHANGED_AT")]
    changed_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl AquaRow {
    pub fn from_entry(entry: &AquaEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            tank_size: entry
                .tank_size_liters
                .map_or_else(|| "-".to_string(), |s| format!("{s}L")),
            changed_at: entry.changed_at.format("%Y-%m-%d %H:%M").to_string(),
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
    pub tank_size_liters: Option<i32>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub changed_at: String,
}

impl From<&AquaEntry> for ListItem {
    fn from(entry: &AquaEntry) -> Self {
        Self {
            id: entry.id.clone(),
            tank_size_liters: entry.tank_size_liters,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            changed_at: entry.changed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
