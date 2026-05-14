use crate::models::{WantEntry, WantStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("wants.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("wants.json")
    }
}

pub fn load_store() -> Result<WantStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(WantStore::default())
    }
}

pub fn save_store(store: &WantStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut WantStore, entry: WantEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut WantStore, name: &str) -> Option<WantEntry> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a WantStore, name: &str) -> Option<&'a WantEntry> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut WantStore, name: &str) -> Option<&'a mut WantEntry> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a WantStore, tag: Option<&'a str>) -> Vec<&'a WantEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}
