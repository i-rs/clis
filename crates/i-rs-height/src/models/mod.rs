use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightRecord {
    pub date: NaiveDate,
    pub height_cm: f64,
    #[serde(default)]
    pub weight_kg: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightStore {
    pub records: BTreeMap<NaiveDate, HeightRecord>,
    #[serde(default)]
    pub target_height: Option<f64>,
}

impl Default for HeightStore {
    fn default() -> Self {
        Self {
            records: BTreeMap::new(),
            target_height: None,
        }
    }
}

impl HeightStore {
    pub fn add_entry(&mut self, record: HeightRecord) {
        self.records.insert(record.date, record);
    }

    pub fn get_entry(&self, date: &NaiveDate) -> Option<&HeightRecord> {
        self.records.get(date)
    }

    pub fn remove_entry(&mut self, date: &NaiveDate) -> Option<HeightRecord> {
        self.records.remove(date)
    }

    pub fn set_target(&mut self, height: f64) {
        self.target_height = Some(height);
    }

    pub fn get_target(&self) -> Option<f64> {
        self.target_height
    }
}
#[derive(Tabled)]
pub struct HeightRow {
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "HEIGHT")]
    height: String,
    #[tabled(rename = "WEIGHT")]
    weight: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

impl HeightRow {
    pub fn from_record(record: &HeightRecord) -> Self {
        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            height: format!("{:.1}", record.height_cm),
            weight: match record.weight_kg {
                Some(w) => format!("{:.1}", w),
                None => "-".to_string(),
            },
            tags: if record.tags.is_empty() {
                "-".to_string()
            } else {
                record.tags.join(", ")
            },
            remark: if record.remark.is_empty() {
                "-".to_string()
            } else {
                record.remark.join(", ")
            },
        }
    }
}
