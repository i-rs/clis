use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightRecord {
    pub id: String,
    pub date: NaiveDate,
    pub weight: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeightStore {
    pub entries: BTreeMap<String, WeightRecord>,
}

impl WeightStore {
    pub fn add_entry(&mut self, record: WeightRecord) {
        self.entries.insert(record.id.clone(), record);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<WeightRecord> {
        self.entries.remove(id)
    }

    #[allow(dead_code)]
    pub fn get_entry(&self, id: &str) -> Option<&WeightRecord> {
        self.entries.get(id)
    }

    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut WeightRecord> {
        self.entries.get_mut(id)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub date: String,
    pub weight: f64,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
}

impl From<&WeightRecord> for ListItem {
    fn from(record: &WeightRecord) -> Self {
        Self {
            id: record.id.clone(),
            date: record.date.format("%Y-%m-%d").to_string(),
            weight: record.weight,
            tags: record.tags.clone(),
            remark: record.remark.clone(),
        }
    }
}

#[derive(Tabled)]
pub struct WeightRow {
    #[tabled(rename = "ID")]
    id: String,
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
            id: record.id[..8].to_string(),
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
