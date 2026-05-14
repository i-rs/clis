use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionRecord {
    pub date: NaiveDate,
    pub left_sphere: Option<f64>,
    pub right_sphere: Option<f64>,
    #[serde(default)]
    pub left_cylinder: Option<f64>,
    #[serde(default)]
    pub right_cylinder: Option<f64>,
    #[serde(default)]
    pub left_axis: Option<i32>,
    #[serde(default)]
    pub right_axis: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct VisionStore {
    pub records: BTreeMap<NaiveDate, VisionRecord>,
}


impl VisionStore {
    pub fn add_entry(&mut self, record: VisionRecord) {
        self.records.insert(record.date, record);
    }

    pub fn remove_entry(&mut self, date: &NaiveDate) -> Option<VisionRecord> {
        self.records.remove(date)
    }

    pub fn get_entry(&self, date: &NaiveDate) -> Option<&VisionRecord> {
        self.records.get(date)
    }

    pub fn get_entry_mut(&mut self, date: &NaiveDate) -> Option<&mut VisionRecord> {
        self.records.get_mut(date)
    }

    pub fn get_recent_records(&self, days: usize) -> Vec<&VisionRecord> {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(days as i64);
        self.records
            .values()
            .filter(|r| r.date >= cutoff)
            .collect()
    }

    pub fn get_all_records(&self) -> Vec<&VisionRecord> {
        self.records.values().collect()
    }

    pub fn records_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_latest_record(&self) -> Option<&VisionRecord> {
        self.records.values().last()
    }

    pub fn get_earliest_record(&self) -> Option<&VisionRecord> {
        self.records.values().next()
    }
}

#[derive(Tabled)]
pub struct VisionRow {
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "L.SPHERE")]
    left_sphere: String,
    #[tabled(rename = "R.SPHERE")]
    right_sphere: String,
    #[tabled(rename = "L.CYL")]
    left_cylinder: String,
    #[tabled(rename = "R.CYL")]
    right_cylinder: String,
    #[tabled(rename = "L.AXIS")]
    left_axis: String,
    #[tabled(rename = "R.AXIS")]
    right_axis: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

impl VisionRow {
    pub fn from_record(record: &VisionRecord) -> Self {
        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            left_sphere: record.left_sphere.map_or_else(|| "-".to_string(), format_sphere),
            right_sphere: record.right_sphere.map_or_else(|| "-".to_string(), format_sphere),
            left_cylinder: record.left_cylinder.map_or_else(|| "-".to_string(), format_sphere),
            right_cylinder: record.right_cylinder.map_or_else(|| "-".to_string(), format_sphere),
            left_axis: record.left_axis.map_or_else(|| "-".to_string(), |a| a.to_string()),
            right_axis: record.right_axis.map_or_else(|| "-".to_string(), |a| a.to_string()),
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

fn format_sphere(value: f64) -> String {
    let sign = if value >= 0.0 { "+" } else { "" };
    format!("{sign}{value:.2}")
}
