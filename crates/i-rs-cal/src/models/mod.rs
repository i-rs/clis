use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalEntry {
    pub id: String,
    pub food_name: String,
    pub calories: i32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    pub date: chrono::NaiveDate,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl CalEntry {
    pub fn new(food_name: String, calories: i32, tags: Vec<String>, remark: Vec<String>, date: chrono::NaiveDate) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, food_name, calories, tags, remark, date, created_at: now }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalStore {
    pub entries: BTreeMap<String, CalEntry>,
}

impl Default for CalStore {
    fn default() -> Self {
        Self { entries: BTreeMap::new() }
    }
}

impl CalStore {
    pub fn add_entry(&mut self, entry: CalEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }
    pub fn remove_entry(&mut self, id: &str) -> Option<CalEntry> {
        self.entries.remove(id)
    }
    pub fn get_entry(&self, id: &str) -> Option<&CalEntry> {
        self.entries.get(id)
    }
    #[allow(dead_code)]
    pub fn get_total_by_date(&self, date: chrono::NaiveDate) -> i32 {
        self.entries.values().filter(|e| e.date == date).map(|e| e.calories).sum()
    }
}

#[derive(Tabled)]
pub struct CalRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "FOOD")]
    food_name: String,
    #[tabled(rename = "CAL")]
    calories: String,
    #[tabled(rename = "DATE")]
    date: String,
}

impl CalRow {
    pub fn from_entry(entry: &CalEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            food_name: entry.food_name.clone(),
            calories: format!("{} kcal", entry.calories),
            date: entry.date.format("%Y-%m-%d").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub id: String,
    pub food_name: String,
    pub calories: i32,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub date: String,
}

impl From<&CalEntry> for ListItem {
    fn from(entry: &CalEntry) -> Self {
        Self {
            id: entry.id.clone(),
            food_name: entry.food_name.clone(),
            calories: entry.calories,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            date: entry.date.format("%Y-%m-%d").to_string(),
        }
    }
}