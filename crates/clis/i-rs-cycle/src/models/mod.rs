use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleEntry {
    pub id: String,
    pub date: chrono::NaiveDate,
    pub event_type: String,
    pub symptoms: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl CycleEntry {
    pub fn new(
        date: chrono::NaiveDate,
        event_type: String,
        symptoms: Vec<String>,
        tags: Vec<String>,
        remark: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            date,
            event_type,
            symptoms,
            tags,
            remark,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CycleStore {
    pub entries: BTreeMap<String, CycleEntry>,
}

impl CycleStore {
    pub fn add_entry(&mut self, entry: CycleEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<CycleEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&CycleEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct CycleRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "EVENT")]
    event_type: String,
    #[tabled(rename = "SYMPTOMS")]
    symptoms: String,
}

impl CycleRow {
    pub fn from_entry(entry: &CycleEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            date: entry.date.format("%Y-%m-%d").to_string(),
            event_type: entry.event_type.clone(),
            symptoms: if entry.symptoms.is_empty() {
                "-".to_string()
            } else {
                entry.symptoms.join(", ")
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub date: String,
    pub event_type: String,
    pub symptoms: Vec<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
}

impl From<&CycleEntry> for ListItem {
    fn from(entry: &CycleEntry) -> Self {
        Self {
            id: entry.id.clone(),
            date: entry.date.format("%Y-%m-%d").to_string(),
            event_type: entry.event_type.clone(),
            symptoms: entry.symptoms.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
        }
    }
}
