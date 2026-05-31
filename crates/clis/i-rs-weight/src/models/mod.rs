use chrono::{DateTime, NaiveDate, Utc};
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
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
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
    pub created_at: String,
    pub updated_at: String,
}

impl From<&WeightRecord> for ListItem {
    fn from(record: &WeightRecord) -> Self {
        Self {
            id: record.id.clone(),
            date: record.date.format("%Y-%m-%d").to_string(),
            weight: record.weight,
            tags: record.tags.clone(),
            remark: record.remark.clone(),
            created_at: record.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: record.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
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
            weight: format!("{:.1} kg", record.weight),
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

// ── IrsTool Spec ──

impl i_rs_core::IrsTool for WeightStore {
    type Entity = WeightRecord;
    type Row = WeightRow;
    type ListItem = ListItem;

    fn tool_name() -> &'static str {
        "weight"
    }
    fn description() -> &'static str {
        "Weight tracking — record body weight over time"
    }
    fn label() -> &'static str {
        "records"
    }
    fn capabilities() -> Vec<i_rs_core::ToolCapability> {
        vec![
            i_rs_core::ToolCapability::DateRange,
            i_rs_core::ToolCapability::Chart,
            i_rs_core::ToolCapability::Stats,
        ]
    }

    fn entries(&self) -> &std::collections::BTreeMap<String, WeightRecord> {
        &self.entries
    }
    fn entries_mut(&mut self) -> &mut std::collections::BTreeMap<String, WeightRecord> {
        &mut self.entries
    }
    fn entity_id(r: &WeightRecord) -> String {
        r.id.clone()
    }
    fn to_row(r: &WeightRecord) -> WeightRow {
        WeightRow::from_record(r)
    }
    fn to_list_item(r: &WeightRecord) -> ListItem {
        ListItem::from(r)
    }
}
