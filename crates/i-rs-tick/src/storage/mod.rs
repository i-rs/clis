use crate::models::{TickEntry, TickStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("ticks.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("ticks.json")
    }
}

pub fn load_store() -> Result<TickStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(TickStore::default())
    }
}

pub fn save_store(store: &TickStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut TickStore, entry: TickEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut TickStore, id: &str) -> Option<TickEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a TickStore, id: &str) -> Option<&'a TickEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a TickStore, tag: Option<&'a str>) -> Vec<&'a TickEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
