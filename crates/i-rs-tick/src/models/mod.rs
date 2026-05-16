use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickEntry {
    pub id: String,
    pub task_name: String,
    pub duration_seconds: i64,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub started_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ended_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl TickEntry {
    pub fn new(
        task_name: String,
        duration_seconds: i64,
        description: Option<String>,
        tags: Vec<String>,
        remark: Vec<String>,
        started_at: DateTime<Utc>,
        ended_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            task_name,
            duration_seconds,
            description,
            tags,
            remark,
            started_at,
            ended_at,
            created_at: now,
        }
    }

    pub fn format_duration(&self) -> String {
        let hours = self.duration_seconds / 3600;
        let minutes = (self.duration_seconds % 3600) / 60;
        let seconds = self.duration_seconds % 60;

        if hours > 0 {
            format!("{hours}h {minutes}m {seconds}s")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TickStore {
    pub entries: BTreeMap<String, TickEntry>,
}

impl TickStore {
    pub fn add_entry(&mut self, entry: TickEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<TickEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&TickEntry> {
        self.entries.get(id)
    }

    pub fn get_total_duration(&self) -> i64 {
        self.entries.values().map(|e| e.duration_seconds).sum()
    }
}

#[derive(Tabled)]
pub struct TickRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "TASK")]
    task_name: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl TickRow {
    pub fn from_entry(entry: &TickEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            task_name: entry.task_name.clone(),
            duration: entry.format_duration(),
            date: entry.started_at.format("%Y-%m-%d").to_string(),
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
    pub task_name: String,
    pub duration_seconds: i64,
    pub duration_str: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub started_at: String,
    pub ended_at: String,
}

impl From<&TickEntry> for ListItem {
    fn from(entry: &TickEntry) -> Self {
        Self {
            id: entry.id.clone(),
            task_name: entry.task_name.clone(),
            duration_seconds: entry.duration_seconds,
            duration_str: entry.format_duration(),
            description: entry.description.clone(),
            tags: entry.tags.clone(),
            started_at: entry.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            ended_at: entry.ended_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_duration_seconds: i64,
    pub total_duration_str: String,
    pub entries_count: usize,
}

impl From<&TickStore> for Summary {
    fn from(store: &TickStore) -> Self {
        let total_seconds = store.get_total_duration();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let duration_str = if hours > 0 {
            format!("{hours}h {minutes}m {seconds}s")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        };

        Self {
            total_duration_seconds: total_seconds,
            total_duration_str: duration_str,
            entries_count: store.entries.len(),
        }
    }
}
