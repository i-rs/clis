use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvEntry {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KvStore {
    pub entries: BTreeMap<String, KvEntry>,
}

impl KvStore {
    pub fn add_entry(&mut self, entry: KvEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }

    pub fn remove_entry(&mut self, key: &str) -> Option<KvEntry> {
        self.entries.remove(key)
    }

    pub fn get_entry(&self, key: &str) -> Option<&KvEntry> {
        self.entries.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut KvEntry> {
        self.entries.get_mut(key)
    }
}

#[derive(Tabled)]
pub struct KvRow {
    #[tabled(rename = "KEY")]
    key: String,
    #[tabled(rename = "VALUE")]
    value: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl KvRow {
    pub fn from_entry(entry: &KvEntry) -> Self {
        Self {
            key: entry.key.clone(),
            value: entry.value.clone(),
            tags: if entry.tags.is_empty() {
                "-".to_string()
            } else {
                entry.tags.join(", ")
            },
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub key: String,
    pub value: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&KvEntry> for ListItem {
    fn from(entry: &KvEntry) -> Self {
        Self {
            key: entry.key.clone(),
            value: entry.value.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KvStats {
    pub total_entries: usize,
    pub total_tags: usize,
    pub total_remarks: usize,
    pub total_value_bytes: usize,
    pub avg_value_bytes: f64,
    pub oldest_entry: Option<String>,
    pub newest_entry: Option<String>,
}
