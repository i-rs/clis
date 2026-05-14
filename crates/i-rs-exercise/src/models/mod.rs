use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseRecord {
    pub name: String,
    pub exercise_type: String,
    pub duration_minutes: u32,
    pub calories: Option<u32>,
    #[serde(default)]
    pub notes: Vec<String>,
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
pub struct ExerciseStore {
    pub records: BTreeMap<String, ExerciseRecord>,
}

impl Default for ExerciseStore {
    fn default() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }
}

impl ExerciseStore {
    #[allow(dead_code)]
    pub fn add_record(&mut self, record: ExerciseRecord) {
        self.records.insert(record.name.clone(), record);
    }

    pub fn remove_record(&mut self, name: &str) -> Option<ExerciseRecord> {
        self.records.remove(name)
    }

    #[allow(dead_code)]
    pub fn get_record(&self, name: &str) -> Option<&ExerciseRecord> {
        self.records.get(name)
    }

    #[allow(dead_code)]
    pub fn get_record_mut(&mut self, name: &str) -> Option<&mut ExerciseRecord> {
        self.records.get_mut(name)
    }

    #[allow(dead_code)]
    pub fn get_all_records(&self) -> Vec<&ExerciseRecord> {
        self.records.values().collect()
    }

    #[allow(dead_code)]
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&ExerciseRecord> {
        self.records
            .values()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .collect()
    }

    #[allow(dead_code)]
    pub fn filter_by_type(&self, exercise_type: &str) -> Vec<&ExerciseRecord> {
        self.records
            .values()
            .filter(|r| r.exercise_type == exercise_type)
            .collect()
    }

    #[allow(dead_code)]
    pub fn total_duration(&self) -> u64 {
        self.records.values().map(|r| r.duration_minutes as u64).sum()
    }

    #[allow(dead_code)]
    pub fn total_calories(&self) -> u64 {
        self.records.values().filter_map(|r| r.calories.map(|c| c as u64)).sum()
    }

    #[allow(dead_code)]
    pub fn records_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Tabled, Serialize)]
pub struct ExerciseRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TYPE")]
    exercise_type: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "CALORIES")]
    calories: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl ExerciseRow {
    pub fn from_record(record: &ExerciseRecord) -> Self {
        Self {
            name: record.name.clone(),
            exercise_type: record.exercise_type.clone(),
            duration: format!("{} min", record.duration_minutes),
            calories: record
                .calories
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string()),
            tags: if record.tags.is_empty() {
                "-".to_string()
            } else {
                record.tags.join(", ")
            },
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ListItem {
    pub name: String,
    pub exercise_type: String,
    pub duration_minutes: u32,
    pub calories: Option<u32>,
    pub notes: Vec<String>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
}

impl From<&ExerciseRecord> for ListItem {
    fn from(record: &ExerciseRecord) -> Self {
        Self {
            name: record.name.clone(),
            exercise_type: record.exercise_type.clone(),
            duration_minutes: record.duration_minutes,
            calories: record.calories,
            notes: record.notes.clone(),
            tags: record.tags.clone(),
            remark: record.remark.clone(),
            created_at: record.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
