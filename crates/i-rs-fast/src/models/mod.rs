use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastEntry {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub target_hours: i32,
    pub actual_hours: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl FastEntry {
    pub fn new(start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>, target_hours: i32, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let actual_hours = end_time.map(|end| (end - start_time).num_hours() as i32);
        Self { id, start_time, end_time, target_hours, actual_hours, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastStore {
    pub entries: BTreeMap<String, FastEntry>,
}

impl Default for FastStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl FastStore {
    pub fn add_entry(&mut self, entry: FastEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<FastEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&FastEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct FastRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "START")]
    start_time: String,
    #[tabled(rename = "TARGET")]
    target: String,
    #[tabled(rename = "ACTUAL")]
    actual: String,
    #[tabled(rename = "STATUS")]
    status: String,
}

impl FastRow {
    pub fn from_entry(entry: &FastEntry) -> Self {
        let actual_str = entry.actual_hours.map(|h| format!("{}h", h)).unwrap_or_else(|| "-".to_string());
        let status = if entry.end_time.is_some() { "DONE" } else { "ACTIVE" };
        Self {
            id: entry.id[..8].to_string(),
            start_time: entry.start_time.format("%Y-%m-%d %H:%M").to_string(),
            target: format!("{}h", entry.target_hours),
            actual: actual_str,
            status: status.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub target_hours: i32,
    pub actual_hours: Option<i32>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
}

impl From<&FastEntry> for ListItem {
    fn from(entry: &FastEntry) -> Self {
        Self {
            id: entry.id.clone(),
            start_time: entry.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            end_time: entry.end_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            target_hours: entry.target_hours,
            actual_hours: entry.actual_hours,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
        }
    }
}
