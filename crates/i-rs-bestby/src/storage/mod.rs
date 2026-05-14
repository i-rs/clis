use crate::models::BestByStore;
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("bestby.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("bestby.json")
    }
}

pub fn load_store() -> Result<BestByStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(BestByStore::default())
    }
}

pub fn save_store(store: &BestByStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut BestByStore, entry: crate::models::Entity) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut BestByStore, name: &str) -> Option<crate::models::Entity> {
    store.remove_entry(name)
}

pub fn get_entry<'a>(store: &'a BestByStore, name: &str) -> Option<&'a crate::models::Entity> {
    store.get_entry(name)
}

pub fn get_entry_mut<'a>(store: &'a mut BestByStore, name: &str) -> Option<&'a mut crate::models::Entity> {
    store.get_entry_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a BestByStore, tag: Option<&'a str>) -> Vec<&'a crate::models::Entity> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
