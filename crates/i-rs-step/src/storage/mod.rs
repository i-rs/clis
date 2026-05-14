use crate::models::{StepEntry, StepStore};
use anyhow::Result;
use chrono::NaiveDate;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("steps.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("steps.json")
    }
}

pub fn load_store() -> Result<StepStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(StepStore::default())
    }
}

pub fn save_store(store: &StepStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut StepStore, entry: StepEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut StepStore, date: &NaiveDate) -> Option<StepEntry> {
    store.remove_entry(date)
}

pub fn get_entry<'a>(store: &'a StepStore, date: &NaiveDate) -> Option<&'a StepEntry> {
    store.get_entry(date)
}

pub fn get_entry_mut<'a>(store: &'a mut StepStore, date: &NaiveDate) -> Option<&'a mut StepEntry> {
    store.get_entry_mut(date)
}
