use crate::models::{WalkdogEntry, WalkdogStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("walkdog.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("walkdog.json")
    }
}

pub fn load_store() -> Result<WalkdogStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(WalkdogStore::default())
    }
}

pub fn save_store(store: &WalkdogStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut WalkdogStore, entry: WalkdogEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut WalkdogStore, id: &str) -> Option<WalkdogEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a WalkdogStore, id: &str) -> Option<&'a WalkdogEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a WalkdogStore, tag: Option<&'a str>) -> Vec<&'a WalkdogEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}