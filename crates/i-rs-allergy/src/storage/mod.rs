use crate::models::{AllergyEntry, AllergyStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("allergies.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("allergies.json")
    }
}

pub fn load_store() -> Result<AllergyStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(AllergyStore::default())
    }
}

pub fn save_store(store: &AllergyStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut AllergyStore, entry: AllergyEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut AllergyStore, id: &str) -> Option<AllergyEntry> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a AllergyStore, id: &str) -> Option<&'a AllergyEntry> {
    store.get_entry(id)
}

pub fn filter_by_tag<'a>(store: &'a AllergyStore, tag: Option<&'a str>) -> Vec<&'a AllergyEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}