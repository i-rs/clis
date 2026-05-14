use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedpetEntry {
    pub id: String,
    pub pet_name: String,
    pub food_type: String,
    pub amount: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub fed_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl FeedpetEntry {
    pub fn new(pet_name: String, food_type: String, amount: String, tags: Vec<String>, remark: Vec<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, pet_name, food_type, amount, fed_at: now, tags, remark, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct FeedpetStore {
    pub entries: BTreeMap<String, FeedpetEntry>,
}


impl FeedpetStore {
    pub fn add_entry(&mut self, entry: FeedpetEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<FeedpetEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&FeedpetEntry> {
        self.entries.get(id)
    }
}

#[derive(Tabled)]
pub struct FeedpetRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "PET")]
    pet_name: String,
    #[tabled(rename = "FOOD")]
    food_type: String,
    #[tabled(rename = "AMOUNT")]
    amount: String,
    #[tabled(rename = "TIME")]
    fed_at: String,
}

impl FeedpetRow {
    pub fn from_entry(entry: &FeedpetEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            pet_name: entry.pet_name.clone(),
            food_type: entry.food_type.clone(),
            amount: entry.amount.clone(),
            fed_at: entry.fed_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub pet_name: String,
    pub food_type: String,
    pub amount: String,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub fed_at: String,
}

impl From<&FeedpetEntry> for ListItem {
    fn from(entry: &FeedpetEntry) -> Self {
        Self {
            id: entry.id.clone(),
            pet_name: entry.pet_name.clone(),
            food_type: entry.food_type.clone(),
            amount: entry.amount.clone(),
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            fed_at: entry.fed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
