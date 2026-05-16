use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealEntry {
    pub id: String,
    pub meal_type: String,
    pub food_items: String,
    #[serde(default)]
    pub calories: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    pub date: NaiveDate,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl MealEntry {
    pub fn new(
        meal_type: String,
        food_items: String,
        calories: Option<i32>,
        tags: Vec<String>,
        remark: Vec<String>,
        date: NaiveDate,
    ) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            meal_type,
            food_items,
            calories,
            tags,
            remark,
            date,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MealStore {
    pub entries: BTreeMap<String, MealEntry>,
}

impl MealStore {
    pub fn add_entry(&mut self, entry: MealEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<MealEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&MealEntry> {
        self.entries.get(id)
    }

    pub fn get_entries_by_date(&self, date: NaiveDate) -> Vec<&MealEntry> {
        let mut entries: Vec<_> = self.entries.values().filter(|e| e.date == date).collect();
        entries.sort_by_key(|e| e.meal_type.clone());
        entries
    }
}

#[derive(Tabled)]
pub struct MealRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "TYPE")]
    meal_type: String,
    #[tabled(rename = "FOOD")]
    food_items: String,
    #[tabled(rename = "CAL")]
    calories: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl MealRow {
    pub fn from_entry(entry: &MealEntry) -> Self {
        Self {
            id: entry.id[..8].to_string(),
            meal_type: entry.meal_type.clone(),
            food_items: entry.food_items.clone(),
            calories: entry
                .calories
                .map_or_else(|| "-".to_string(), |c| c.to_string()),
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
    pub meal_type: String,
    pub food_items: String,
    pub calories: Option<i32>,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub date: String,
    pub created_at: String,
}

impl From<&MealEntry> for ListItem {
    fn from(entry: &MealEntry) -> Self {
        Self {
            id: entry.id.clone(),
            meal_type: entry.meal_type.clone(),
            food_items: entry.food_items.clone(),
            calories: entry.calories,
            tags: entry.tags.clone(),
            remark: entry.remark.clone(),
            date: entry.date.format("%Y-%m-%d").to_string(),
            created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
