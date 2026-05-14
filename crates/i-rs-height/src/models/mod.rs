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
    #[allow(dead_code)]
    pub fn add_record(&mut self, record: HeightRecord) {
        self.records.insert(record.date, record);
    }

    pub fn remove_record(&mut self, date: &NaiveDate) -> Option<HeightRecord> {
        self.records.remove(date)
    }

    #[allow(dead_code)]
    pub fn get_record(&self, date: &NaiveDate) -> Option<&HeightRecord> {
        self.records.get(date)
    }

    #[allow(dead_code)]
    pub fn get_record_mut(&mut self, date: &NaiveDate) -> Option<&mut HeightRecord> {
        self.records.get_mut(date)
    }

    #[allow(dead_code)]
    pub fn get_recent_records(&self, days: usize) -> Vec<&HeightRecord> {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(days as i64);
        self.records
            .values()
            .filter(|r| r.date >= cutoff)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_all_records(&self) -> Vec<&HeightRecord> {
        self.records.values().collect()
    }

    #[allow(dead_code)]
    pub fn min_height(&self) -> Option<f64> {
        self.records.values().map(|r| r.height_cm).reduce(f64::min)
    }

    #[allow(dead_code)]
    pub fn max_height(&self) -> Option<f64> {
        self.records.values().map(|r| r.height_cm).reduce(f64::max)
    }

    #[allow(dead_code)]
    pub fn avg_height(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.values().map(|r| r.height_cm).sum();
        Some(sum / self.records.len() as f64)
    }

    #[allow(dead_code)]
    pub fn total_height_change(&self) -> Option<f64> {
        let records: Vec<_> = self.records.values().collect();
        if records.len() < 2 {
            return None;
        }
        let first = records.first().unwrap().height_cm;
        let last = records.last().unwrap().height_cm;
        Some(last - first)
    }

    #[allow(dead_code)]
    pub fn records_count(&self) -> usize {
        self.records.len()
    }

    #[allow(dead_code)]
    pub fn set_target(&mut self, height: f64) {
        self.target_height = Some(height);
    }

    #[allow(dead_code)]
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
