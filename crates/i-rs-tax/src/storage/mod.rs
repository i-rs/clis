use crate::models::{TaxRecord, TaxStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    if let Some(config_dir) = std::env::var_os("CONFIG_DIR") {
        PathBuf::from(config_dir)
    } else {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("i-rs")
    }
}

pub fn get_file_path() -> PathBuf {
    get_config_dir().join("tax.json")
}

pub fn load_store() -> Result<TaxStore> {
    let path = get_file_path();
    if !path.exists() {
        return Ok(TaxStore::new());
    }
    let content = std::fs::read_to_string(&path)?;
    let store: TaxStore = serde_json::from_str(&content)?;
    Ok(store)
}

pub fn save_store(store: &TaxStore) -> Result<()> {
    let path = get_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut TaxStore, entry: TaxRecord) {
    store.entries.insert(entry.name.clone(), entry);
}

pub fn remove_entry(store: &mut TaxStore, name: &str) -> Option<TaxRecord> {
    store.entries.remove(name)
}

#[allow(dead_code)]
pub fn get_entry<'a>(store: &'a TaxStore, name: &str) -> Option<&'a TaxRecord> {
    store.entries.get(name)
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn get_entry_mut<'a>(store: &'a mut TaxStore, name: &str) -> Option<&'a mut TaxRecord> {
    store.entries.get_mut(name)
}

#[allow(dead_code)]
pub fn filter_by_tag<'a>(store: &'a TaxStore, tag: Option<&'a str>) -> Vec<&'a TaxRecord> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag.contains(t)))
            .collect(),
        None => store.entries.values().collect(),
    }
}
