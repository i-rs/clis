use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEntry {
    pub id: String,
    pub steps: i32,
    pub distance: Option<f64>,
    pub date: NaiveDate,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl StepEntry {
    pub fn new(steps: i32, distance: Option<f64>, date: NaiveDate, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            steps,
            distance,
            date,
            tags,
            remark,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct StepStore {
    pub entries: BTreeMap<NaiveDate, StepEntry>,
}


impl StepStore {
    pub fn add_entry(&mut self, entry: StepEntry) {
        self.entries.insert(entry.date, entry);
    }

    pub fn remove_entry(&mut self, date: &NaiveDate) -> Option<StepEntry> {
        self.entries.remove(date)
    }

    pub fn get_entry(&self, date: &NaiveDate) -> Option<&StepEntry> {
        self.entries.get(date)
    }

    pub fn get_entry_mut(&mut self, date: &NaiveDate) -> Option<&mut StepEntry> {
        self.entries.get_mut(date)
    }

    pub fn get_total_steps(&self) -> i32 {
        self.entries.values().map(|e| e.steps).sum()
    }

    pub fn get_total_distance(&self) -> Option<f64> {
        let sum: f64 = self.entries.values().filter_map(|e| e.distance).sum();
        if sum > 0.0 { Some(sum) } else { None }
    }
}

#[derive(Tabled)]
pub struct StepRow {
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "STEPS")]
    steps: String,
    #[tabled(rename = "DISTANCE")]
    distance: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl StepRow {
    pub fn from_entry(entry: &StepEntry) -> Self {
        Self {
            date: entry.date.format("%Y-%m-%d").to_string(),
            steps: format!("{}", entry.steps),
            distance: entry.distance.map_or_else(|| "-".to_string(), |d| format!("{d:.1}km")),
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
    pub date: String,
    pub steps: i32,
    pub distance: Option<f64>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
}

impl From<&StepEntry> for ListItem {
    fn from(entry: &StepEntry) -> Self {
        Self {
            date: entry.date.format("%Y-%m-%d").to_string(),
            steps: entry.steps,
            distance: entry.distance,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_steps: i32,
    pub total_distance: Option<f64>,
    pub days_count: usize,
}

impl From<&StepStore> for Summary {
    fn from(store: &StepStore) -> Self {
        Self {
            total_steps: store.get_total_steps(),
            total_distance: store.get_total_distance(),
            days_count: store.entries.len(),
        }
    }
}
