use crate::models::{MealEntry, MealStore};
use anyhow::Result;
use chrono::NaiveDate;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("meals.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("meals.json")
    }
}

pub fn load_store() -> Result<MealStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(MealStore::default())
    }
}

pub fn save_store(store: &MealStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut MealStore, entry: MealEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut MealStore, id: &str) -> Option<MealEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a MealStore, id: &str) -> Option<&'a MealEntry> {
    store.get_entry(id)
}

pub fn get_entries_by_date(store: &MealStore, date: NaiveDate) -> Vec<&MealEntry> {
    store.get_entries_by_date(date)
}
