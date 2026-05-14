use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterEntry {
    pub id: String,
    pub appliance_name: String,
    pub filter_type: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub cleaned_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl FilterEntry {
    pub fn new(appliance_name: String, filter_type: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, appliance_name, filter_type, cleaned_at: now, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterStore {
    pub entries: BTreeMap<String, FilterEntry>,
}

impl Default for FilterStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl FilterStore {
    pub fn add_entry(&mut self, entry: FilterEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<FilterEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&FilterEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct FilterRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "APPLIANCE")]
    appliance_name: String,
    #[tabled(rename = "FILTER_TYPE")]
    filter_type: String,
    #[tabled(rename = "CLEANED_AT")]
    cleaned_at: String,
}

impl FilterRow {
    pub fn from_entry(entry: &FilterEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            appliance_name: entry.appliance_name.clone(),
            filter_type: entry.filter_type.clone(),
            cleaned_at: entry.cleaned_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub appliance_name: String,
    pub filter_type: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub cleaned_at: String,
}

impl From<&FilterEntry> for ListItem {
    fn from(entry: &FilterEntry) -> Self {
        Self {
            id: entry.id.clone(),
            appliance_name: entry.appliance_name.clone(),
            filter_type: entry.filter_type.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            cleaned_at: entry.cleaned_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
