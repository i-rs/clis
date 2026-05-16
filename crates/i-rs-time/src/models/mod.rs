use chrono::{DateTime, NaiveDate, Utc};
use i_rs_core::storage::HasTags;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: String,
    pub name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: i64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl HasTags for TimeEntry {
    fn tags(&self) -> &[String] {
        &self.tags
    }
}

impl TimeEntry {
    pub const fn is_running(&self) -> bool {
        self.end_time.is_none()
    }

    #[allow(dead_code)]
    pub fn calculate_duration(&self) -> i64 {
        if let Some(end) = self.end_time {
            (end - self.start_time).num_minutes()
        } else {
            (Utc::now() - self.start_time).num_minutes()
        }
    }

    #[allow(dead_code)]
    pub fn get_date(&self) -> NaiveDate {
        self.start_time.date_naive()
    }

    pub fn stop(&mut self) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.duration_minutes = (now - self.start_time).num_minutes();
        self.updated_at = now;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeStore {
    pub entries: BTreeMap<String, TimeEntry>,
    pub active_entry_id: Option<String>,
}

impl TimeStore {
    pub fn add_entry(&mut self, entry: TimeEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<TimeEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&TimeEntry> {
        self.entries.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut TimeEntry> {
        self.entries.get_mut(id)
    }

    pub fn get_active_entry(&self) -> Option<&TimeEntry> {
        self.active_entry_id
            .as_ref()
            .and_then(|id| self.entries.get(id))
    }

    #[allow(dead_code)]
    pub fn get_active_entry_mut(&mut self) -> Option<&mut TimeEntry> {
        if let Some(ref id) = self.active_entry_id {
            self.entries.get_mut(id)
        } else {
            None
        }
    }

    pub fn get_entries_by_date(&self, date: NaiveDate) -> Vec<&TimeEntry> {
        self.entries
            .values()
            .filter(|e| e.start_time.date_naive() == date)
            .collect()
    }

    pub fn get_entries_in_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<&TimeEntry> {
        self.entries
            .values()
            .filter(|e| {
                let entry_date = e.start_time.date_naive();
                entry_date >= start && entry_date <= end
            })
            .collect()
    }

    pub fn get_all_entries(&self) -> Vec<&TimeEntry> {
        self.entries.values().collect()
    }

    #[allow(dead_code)]
    pub fn get_completed_entries(&self) -> Vec<&TimeEntry> {
        self.entries
            .values()
            .filter(|e| e.end_time.is_some())
            .collect()
    }

    #[allow(dead_code)]
    pub fn total_minutes(&self, entries: &[&TimeEntry]) -> i64 {
        entries.iter().map(|e| e.duration_minutes).sum()
    }
}

#[derive(Tabled)]
pub struct TimeEntryRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "START")]
    start_time: String,
    #[tabled(rename = "END")]
    end_time: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl TimeEntryRow {
    pub fn from_entry(entry: &TimeEntry) -> Self {
        let duration = if entry.duration_minutes >= 60 {
            format!(
                "{}h {}m",
                entry.duration_minutes / 60,
                entry.duration_minutes % 60
            )
        } else {
            format!("{}m", entry.duration_minutes)
        };

        Self {
            id: entry.id[..8].to_string(),
            name: entry.name.clone(),
            start_time: entry.start_time.format("%Y-%m-%d %H:%M").to_string(),
            end_time: entry.end_time.map_or_else(
                || "Running...".to_string(),
                |t| t.format("%Y-%m-%d %H:%M").to_string(),
            ),
            duration,
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
    pub name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_running: bool,
}

impl From<&TimeEntry> for ListItem {
    fn from(entry: &TimeEntry) -> Self {
        Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            start_time: entry.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            end_time: entry
                .end_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            duration_minutes: entry.duration_minutes,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            is_running: entry.is_running(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StatsData {
    pub date: String,
    pub total_minutes: i64,
    pub entries_count: usize,
    pub entries: Vec<ListItem>,
}

#[derive(Debug, Serialize)]
pub struct ReportData {
    pub start_date: String,
    pub end_date: String,
    pub total_minutes: i64,
    pub total_hours: f64,
    pub entries_count: usize,
    pub daily_breakdown: Vec<StatsData>,
}
