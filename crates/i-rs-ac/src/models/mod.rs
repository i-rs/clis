use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcEntry {
    pub id: String,
    pub location: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub cleaned_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl AcEntry {
    pub fn new(location: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, location, cleaned_at: now, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcStore {
    pub entries: BTreeMap<String, AcEntry>,
}

impl Default for AcStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl AcStore {
    pub fn add_entry(&mut self, entry: AcEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<AcEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&AcEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct AcRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "LOCATION")]
    location: String,
    #[tabled(rename = "CLEANED_AT")]
    cleaned_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl AcRow {
    pub fn from_entry(entry: &AcEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            location: entry.location.clone(),
            cleaned_at: entry.cleaned_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: if entry.tags.is_empty() { "-".to_string() } else { entry.tags.join(", ") },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub location: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub cleaned_at: String,
}

impl From<&AcEntry> for ListItem {
    fn from(entry: &AcEntry) -> Self {
        Self {
            id: entry.id.clone(),
            location: entry.location.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            cleaned_at: entry.cleaned_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
