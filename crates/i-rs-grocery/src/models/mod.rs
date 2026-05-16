use chrono::{DateTime, Utc};
use i_rs_core::storage::HasTags;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroceryItem {
    pub name: String,
    pub quantity: i32,
    pub unit: String,
    pub purchased: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl HasTags for GroceryItem {
    fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroceryStore {
    pub entries: BTreeMap<String, GroceryItem>,
}

impl GroceryStore {
    pub fn add_entry(&mut self, entry: GroceryItem) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<GroceryItem> {
        self.entries.remove(name)
    }

    pub fn get_entry(&self, name: &str) -> Option<&GroceryItem> {
        self.entries.get(name)
    }

    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut GroceryItem> {
        self.entries.get_mut(name)
    }
}

#[derive(Tabled)]
pub struct GroceryRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "QTY")]
    quantity: String,
    #[tabled(rename = "UNIT")]
    unit: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl GroceryRow {
    pub fn from_item(item: &GroceryItem) -> Self {
        Self {
            name: item.name.clone(),
            quantity: item.quantity.to_string(),
            unit: item.unit.clone(),
            status: if item.purchased {
                "✅ Purchased".to_string()
            } else {
                "🔄 Needed".to_string()
            },
            tags: if item.tags.is_empty() {
                "-".to_string()
            } else {
                item.tags.join(", ")
            },
            updated_at: item.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ListItem {
    pub name: String,
    pub quantity: i32,
    pub unit: String,
    pub purchased: bool,
    pub tags: Vec<String>,
    pub remark: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&GroceryItem> for ListItem {
    fn from(item: &GroceryItem) -> Self {
        Self {
            name: item.name.clone(),
            quantity: item.quantity,
            unit: item.unit.clone(),
            purchased: item.purchased,
            tags: item.tags.clone(),
            remark: item.remark.clone(),
            created_at: item.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: item.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}
