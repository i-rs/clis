use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkdogEntry {
    pub id: String,
    pub dog_name: String,
    pub duration_minutes: i32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub walked_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl WalkdogEntry {
    pub fn new(dog_name: String, duration_minutes: i32, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, dog_name, duration_minutes, walked_at: now, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct WalkdogStore {
    pub entries: BTreeMap<String, WalkdogEntry>,
}


impl WalkdogStore {
    pub fn add_entry(&mut self, entry: WalkdogEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<WalkdogEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&WalkdogEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct WalkdogRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "DOG")]
    dog_name: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "TIME")]
    walked_at: String,
}

impl WalkdogRow {
    pub fn from_entry(entry: &WalkdogEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            dog_name: entry.dog_name.clone(),
            duration: format!("{} min", entry.duration_minutes),
            walked_at: entry.walked_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub dog_name: String,
    pub duration_minutes: i32,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub walked_at: String,
}

impl From<&WalkdogEntry> for ListItem {
    fn from(entry: &WalkdogEntry) -> Self {
        Self {
            id: entry.id.clone(),
            dog_name: entry.dog_name.clone(),
            duration_minutes: entry.duration_minutes,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            walked_at: entry.walked_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
