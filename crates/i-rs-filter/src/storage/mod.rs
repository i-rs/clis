use crate::models::{FilterEntry, FilterStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("filters.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("filters.json")
    }
}

pub fn load_store() -> Result<FilterStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(FilterStore::default())
    }
}

pub fn save_store(store: &FilterStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut FilterStore, entry: FilterEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut FilterStore, id: &str) -> Option<FilterEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a FilterStore, id: &str) -> Option<&'a FilterEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a FilterStore, tag: Option<&'a str>) -> Vec<&'a FilterEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}