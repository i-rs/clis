use chrono::{DateTime, Utc};
use i_rs_core::storage::HasTags;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepRecord {
    pub id: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub bedtime: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub wake_time: DateTime<Utc>,
    pub quality: i32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl HasTags for SleepRecord {
    fn tags(&self) -> &[String] {
        &self.tags
    }
}

impl SleepRecord {
    pub fn duration_hours(&self) -> f64 {
        let duration = self.wake_time - self.bedtime;
        duration.num_seconds() as f64 / 3600.0
    }

    pub fn quality_label(&self) -> String {
        match self.quality {
            1 => "😴 Awful".to_string(),
            2 => "😪 Poor".to_string(),
            3 => "😌 Fair".to_string(),
            4 => "😊 Good".to_string(),
            5 => "😁 Excellent".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepStore {
    pub entries: BTreeMap<String, SleepRecord>,
}

impl Default for SleepStore {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl SleepStore {
    pub fn add_entry(&mut self, entry: SleepRecord) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<SleepRecord> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&SleepRecord> {
        self.entries.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut SleepRecord> {
        self.entries.get_mut(id)
    }
}

#[derive(Tabled)]
pub struct SleepRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "BEDTIME")]
    bedtime: String,
    #[tabled(rename = "WAKE")]
    wake_time: String,
    #[tabled(rename = "HOURS")]
    duration: String,
    #[tabled(rename = "QUALITY")]
    quality: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl SleepRow {
    pub fn from_record(record: &SleepRecord) -> Self {
        Self {
            id: record.id.clone(),
            bedtime: record.bedtime.format("%Y-%m-%d %H:%M").to_string(),
            wake_time: record.wake_time.format("%H:%M").to_string(),
            duration: format!("{:.1}h", record.duration_hours()),
            quality: record.quality_label(),
            tags: if record.tags.is_empty() { "-".to_string() } else { record.tags.join(", ") },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub bedtime: String,
    pub wake_time: String,
    pub duration_hours: f64,
    pub quality: i32,
    pub quality_label: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&SleepRecord> for ListItem {
    fn from(record: &SleepRecord) -> Self {
        Self {
            id: record.id.clone(),
            bedtime: record.bedtime.format("%Y-%m-%d %H:%M:%S").to_string(),
            wake_time: record.wake_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            duration_hours: record.duration_hours(),
            quality: record.quality,
            quality_label: record.quality_label(),
            tags: record.tags.clone(),
            remark: record.remark.clone(),
            created_at: record.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct SleepStats {
    pub total_records: usize,
    pub avg_duration: f64,
    pub avg_quality: f64,
    pub min_duration: f64,
    pub max_duration: f64,
}

impl SleepStats {
    pub fn from_records(records: &[&SleepRecord]) -> Self {
        if records.is_empty() {
            return Self {
                total_records: 0,
                avg_duration: 0.0,
                avg_quality: 0.0,
                min_duration: 0.0,
                max_duration: 0.0,
            };
        }

        let total_duration: f64 = records.iter().map(|r| r.duration_hours()).sum();
        let total_quality: i32 = records.iter().map(|r| r.quality).sum();
        let durations: Vec<f64> = records.iter().map(|r| r.duration_hours()).collect();

        Self {
            total_records: records.len(),
            avg_duration: total_duration / records.len() as f64,
            avg_quality: total_quality as f64 / records.len() as f64,
            min_duration: *durations.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max_duration: *durations.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        }
    }
}