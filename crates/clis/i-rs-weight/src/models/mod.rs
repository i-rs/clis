use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightRecord {
    pub date: NaiveDate,
    pub weight: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeightStore {
    pub records: BTreeMap<NaiveDate, WeightRecord>,
}

impl WeightStore {
    pub fn add_entry(&mut self, record: WeightRecord) {
        self.records.insert(record.date, record);
    }

    pub fn remove_entry(&mut self, date: &NaiveDate) -> Option<WeightRecord> {
        self.records.remove(date)
    }

    #[allow(dead_code)]
    pub fn get_entry(&self, date: &NaiveDate) -> Option<&WeightRecord> {
        self.records.get(date)
    }

    pub fn get_entry_mut(&mut self, date: &NaiveDate) -> Option<&mut WeightRecord> {
        self.records.get_mut(date)
    }
}

#[derive(Tabled)]
pub struct WeightRow {
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "WEIGHT")]
    weight: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

impl WeightRow {
    pub fn from_record(record: &WeightRecord) -> Self {
        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            weight: format!("{:.1}", record.weight),
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
