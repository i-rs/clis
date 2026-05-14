use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetEntry {
    pub id: String,
    pub sheet_type: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub changed_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl SheetEntry {
    pub fn new(sheet_type: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, sheet_type, changed_at: now, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetStore {
    pub entries: BTreeMap<String, SheetEntry>,
}

impl Default for SheetStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl SheetStore {
    pub fn add_entry(&mut self, entry: SheetEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<SheetEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&SheetEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct SheetRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "TYPE")]
    sheet_type: String,
    #[tabled(rename = "CHANGED")]
    changed_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl SheetRow {
    pub fn from_entry(entry: &SheetEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            sheet_type: entry.sheet_type.clone(),
            changed_at: entry.changed_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: if entry.tags.is_empty() { "-".to_string() } else { entry.tags.join(", ") },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub sheet_type: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub changed_at: String,
}

impl From<&SheetEntry> for ListItem {
    fn from(entry: &SheetEntry) -> Self {
        Self {
            id: entry.id.clone(),
            sheet_type: entry.sheet_type.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            changed_at: entry.changed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
