use crate::models::{KvEntry, KvStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("kv.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("kv.json")
    }
}

pub fn load_store() -> Result<KvStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(KvStore::default())
    }
}

pub fn save_store(store: &KvStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut KvStore, entry: KvEntry) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut KvStore, name: &str) -> Option<KvEntry> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a KvStore, name: &str) -> Option<&'a KvEntry> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut KvStore, name: &str) -> Option<&'a mut KvEntry> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a KvStore, tag: Option<&'a str>) -> Vec<&'a KvEntry> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
