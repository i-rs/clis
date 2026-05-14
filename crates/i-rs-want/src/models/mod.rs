use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantEntry {
    pub name: String,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub priority: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(default)]
    pub is_done: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl WantEntry {
    pub fn new(
        name: String,
        url: Option<String>,
        price: Option<f64>,
        currency: Option<String>,
        priority: String,
        tags: Vec<String>,
        remark: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            name,
            url,
            price,
            currency,
            priority,
            tags,
            remark,
            is_done: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantStore {
    pub entries: BTreeMap<String, WantEntry>,
}

impl Default for WantStore {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl WantStore {
    pub fn add_entry(&mut self, entry: WantEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<WantEntry> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&WantEntry> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut WantEntry> {
        self.entries.get_mut(name)
    }
}

#[derive(Tabled)]
pub struct WantRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PRICE")]
    price: String,
    #[tabled(rename = "PRIORITY")]
    priority: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl WantRow {
    pub fn from_entry(entry: &WantEntry) -> Self {
        let price_str = match (entry.price, &entry.currency) {
            (Some(p), Some(c)) => format!("{:.2} {}", p, c),
            (Some(p), None) => format!("{:.2}", p),
            (None, _) => "-".to_string(),
        };

        Self {
            name: entry.name.clone(),
            price: price_str,
            priority: entry.priority.clone(),
            status: if entry.is_done { "DONE" } else { "PENDING" }.to_string(),
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
    pub name: String,
    pub url: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub priority: String,
    pub is_done: bool,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&WantEntry> for ListItem {
    fn from(entry: &WantEntry) -> Self {
        Self {
            name: entry.name.clone(),
            url: entry.url.clone(),
            price: entry.price,
            currency: entry.currency.clone(),
            priority: entry.priority.clone(),
            is_done: entry.is_done,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
