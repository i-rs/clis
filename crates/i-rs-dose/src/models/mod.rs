use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoseEntry {
    pub id: String,
    pub medicine_name: String,
    pub dosage: String,
    pub unit: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub taken_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl DoseEntry {
    pub fn new(medicine_name: String, dosage: String, unit: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            medicine_name,
            dosage,
            unit,
            tags,
            remark,
            taken_at: now,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoseStore {
    pub entries: BTreeMap<String, DoseEntry>,
}

impl Default for DoseStore {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl DoseStore {
    pub fn add_entry(&mut self, entry: DoseEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<DoseEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&DoseEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct DoseRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "MEDICINE")]
    medicine_name: String,
    #[tabled(rename = "DOSAGE")]
    dosage: String,
    #[tabled(rename = "TIME")]
    taken_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl DoseRow {
    pub fn from_entry(entry: &DoseEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            medicine_name: entry.medicine_name.clone(),
            dosage: format!("{} {}", entry.dosage, entry.unit),
            taken_at: entry.taken_at.format("%Y-%m-%d %H:%M").to_string(),
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
    pub id: String,
    pub medicine_name: String,
    pub dosage: String,
    pub unit: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub taken_at: String,
    pub created_at: String,
}

impl From<&DoseEntry> for ListItem {
    fn from(entry: &DoseEntry) -> Self {
        Self {
            id: entry.id.clone(),
            medicine_name: entry.medicine_name.clone(),
            dosage: entry.dosage.clone(),
            unit: entry.unit.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            taken_at: entry.taken_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
