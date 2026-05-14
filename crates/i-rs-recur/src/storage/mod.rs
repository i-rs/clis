use crate::models::{RecurEntry, RecurStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("recur.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("recur.json")
    }
}

pub fn load_store() -> Result<RecurStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(RecurStore::default())
    }
}

pub fn save_store(store: &RecurStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut RecurStore, entry: RecurEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut RecurStore, name: &str) -> Option<RecurEntry> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a RecurStore, name: &str) -> Option<&'a RecurEntry> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut RecurStore, name: &str) -> Option<&'a mut RecurEntry> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a RecurStore, tag: Option<&'a str>) -> Vec<&'a RecurEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
