use crate::models::{DeployRecord, DeployStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("deploy.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("deploy.json")
    }
}

pub fn load_store() -> Result<DeployStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(DeployStore::default())
    }
}

pub fn save_store(store: &DeployStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_entry(store: &mut DeployStore, entry: DeployRecord) {
    store.add_entry(entry);
}

pub fn remove_entry(store: &mut DeployStore, id: &str) -> Option<DeployRecord> {
    store.remove_entry(id)
}

pub fn get_entry<'a>(store: &'a DeployStore, id: &str) -> Option<&'a DeployRecord> {
    store.get_entry(id)
}

#[allow(dead_code)]
pub fn get_entry_mut<'a>(store: &'a mut DeployStore, id: &str) -> Option<&'a mut DeployRecord> {
    store.get_entry_mut(id)
}

pub fn filter_by_project<'a>(store: &'a DeployStore, project: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match project {
        Some(p) => store
            .entries
            .values()
            .filter(|e| e.project == p)
            .collect(),
        None => store.entries.values().collect(),
    }
}

pub fn filter_by_environment<'a>(store: &'a DeployStore, env: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match env {
        Some(e) => store
            .entries
            .values()
            .filter(|r| r.environment == e)
            .collect(),
        None => store.entries.values().collect(),
    }
}

pub fn filter_by_tag<'a>(store: &'a DeployStore, tag: Option<&'a str>) -> Vec<&'a DeployRecord> {
    match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .collect(),
        None => store.entries.values().collect(),
    }
}
