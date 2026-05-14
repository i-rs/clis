use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    pub name: String,
    pub key_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct KeyStore {
    pub entries: BTreeMap<String, KeyEntry>,
}


impl KeyStore {
    pub fn add_entry(&mut self, entry: KeyEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<KeyEntry> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&KeyEntry> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut KeyEntry> {
        self.entries.get_mut(name)
    }
}

#[derive(Tabled)]
pub struct KeyRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TYPE")]
    key_type: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl KeyRow {
    pub fn from_entry(entry: &KeyEntry) -> Self {
        Self {
            name: entry.name.clone(),
            key_type: entry.key_type.clone(),
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
    pub name: String,
    pub key_type: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&KeyEntry> for ListItem {
    fn from(entry: &KeyEntry) -> Self {
        Self {
            name: entry.name.clone(),
            key_type: entry.key_type.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
