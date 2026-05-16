use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitEntry {
    pub id: String,
    pub duration_minutes: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl SitEntry {
    pub fn new(
        duration_minutes: i32,
        started_at: DateTime<Utc>,
        ended_at: DateTime<Utc>,
        tags: Vec<String>,
        remark: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            duration_minutes,
            started_at,
            ended_at,
            tags,
            remark,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SitStore {
    pub entries: BTreeMap<String, SitEntry>,
}

#[allow(dead_code)]
impl SitStore {
    pub fn add_entry(&mut self, entry: SitEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<SitEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&SitEntry> {
        self.entries.get(id)
    }
    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut SitEntry> {
        self.entries.get_mut(id)
    }
}

#[derive(Tabled)]
pub struct SitRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl SitRow {
    pub fn from_entry(entry: &SitEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            duration: format!("{} min", entry.duration_minutes),
            date: entry.started_at.format("%Y-%m-%d %H:%M").to_string(),
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
    pub duration_minutes: i32,
    pub started_at: String,
    pub ended_at: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
}

impl From<&SitEntry> for ListItem {
    fn from(entry: &SitEntry) -> Self {
        Self {
            id: entry.id.clone(),
            duration_minutes: entry.duration_minutes,
            started_at: entry.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            ended_at: entry.ended_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
        }
    }
}
