use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightRecord {
    pub date: NaiveDate,
    pub weight: f64,
    #[serde(default)]
    pub remark: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightStore {
    pub records: BTreeMap<NaiveDate, WeightRecord>,
}

impl Default for WeightStore {
    fn default() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }
}

impl WeightStore {
    #[allow(dead_code)]
    pub fn add_record(&mut self, record: WeightRecord) {
        self.records.insert(record.date, record);
    }

    pub fn remove_record(&mut self, date: &NaiveDate) -> Option<WeightRecord> {
        self.records.remove(date)
    }

    #[allow(dead_code)]
    pub fn get_record(&self, date: &NaiveDate) -> Option<&WeightRecord> {
        self.records.get(date)
    }

    #[allow(dead_code)]
    pub fn get_record_mut(&mut self, date: &NaiveDate) -> Option<&mut WeightRecord> {
        self.records.get_mut(date)
    }

    #[allow(dead_code)]
    pub fn get_recent_records(&self, days: usize) -> Vec<&WeightRecord> {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(days as i64);
        self.records
            .values()
            .filter(|r| r.date >= cutoff)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_all_records(&self) -> Vec<&WeightRecord> {
        self.records.values().collect()
    }

    pub fn min_weight(&self) -> Option<f64> {
        self.records.values().map(|r| r.weight).reduce(f64::min)
    }

    pub fn max_weight(&self) -> Option<f64> {
        self.records.values().map(|r| r.weight).reduce(f64::max)
    }

    pub fn avg_weight(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.values().map(|r| r.weight).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn total_change(&self) -> Option<f64> {
        let records: Vec<_> = self.records.values().collect();
        if records.len() < 2 {
            return None;
        }
        let first = records.first().unwrap().weight;
        let last = records.last().unwrap().weight;
        Some(last - first)
    }

    #[allow(dead_code)]
    pub fn records_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Tabled)]
pub struct WeightRow {
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "WEIGHT")]
    weight: String,
    #[tabled(rename = "REMARK")]
    remark: String,
}

impl WeightRow {
    pub fn from_record(record: &WeightRecord) -> Self {
        Self {
            date: record.date.format("%Y-%m-%d").to_string(),
            weight: format!("{:.1}", record.weight),
            remark: if record.remark.is_empty() {
                "-".to_string()
            } else {
                record.remark.join(", ")
            },
        }
    }
}
