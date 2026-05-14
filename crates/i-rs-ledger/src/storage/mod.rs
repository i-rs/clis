use crate::models::{LedgerEntry, LedgerStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("ledger.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("ledger.json")
    }
}

pub fn load_store() -> Result<LedgerStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(LedgerStore::default())
    }
}

pub fn save_store(store: &LedgerStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut LedgerStore, entry: LedgerEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut LedgerStore, id: &str) -> Option<LedgerEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a LedgerStore, id: &str) -> Option<&'a LedgerEntry> {
    store.get_entry(id)
}

pub fn filter_by_category<'a>(store: &'a LedgerStore, category: Option<&'a str>) -> Vec<&'a LedgerEntry> {
    match category {
        Some(c) => store.get_entries_by_category(c),
        None => store.entries.values().collect(),
    }
}
