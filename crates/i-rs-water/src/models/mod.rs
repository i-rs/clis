use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterEntry {
    pub id: String,
    pub amount_ml: i32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub drank_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl WaterEntry {
    pub fn new(amount_ml: i32, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            amount_ml,
            tags,
            remark,
            drank_at: now,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct WaterStore {
    pub entries: BTreeMap<String, WaterEntry>,
}


#[allow(dead_code)]
impl WaterStore {
    pub fn add_entry(&mut self, entry: WaterEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<WaterEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&WaterEntry> {
        self.entries.get(id)
    }
    pub fn get_total_today(&self) -> i32 {
        let today = Utc::now().date_naive();
        self.entries.values()
            .filter(|e| e.drank_at.date_naive() == today)
            .map(|e| e.amount_ml)
            .sum()
    }
}

#[derive(Tabled)]
pub struct WaterRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "AMOUNT")]
    amount: String,
    #[tabled(rename = "TIME")]
    drank_at: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl WaterRow {
    pub fn from_entry(entry: &WaterEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            amount: format!("{} ml", entry.amount_ml),
            drank_at: entry.drank_at.format("%H:%M").to_string(),
            tags: if entry.tags.is_empty() { "-".to_string() } else { entry.tags.join(", ") },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub amount_ml: i32,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub drank_at: String,
}

impl From<&WaterEntry> for ListItem {
    fn from(entry: &WaterEntry) -> Self {
        Self {
            id: entry.id.clone(),
            amount_ml: entry.amount_ml,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            drank_at: entry.drank_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Summary {
    pub today_total_ml: i32,
    pub today_count: usize,
}

impl From<&WaterStore> for Summary {
    fn from(store: &WaterStore) -> Self {
        let today = Utc::now().date_naive();
        let entries: Vec<_> = store.entries.values().filter(|e| e.drank_at.date_naive() == today).collect();
        Self {
            today_total_ml: entries.iter().map(|e| e.amount_ml).sum(),
            today_count: entries.len(),
        }
    }
}
