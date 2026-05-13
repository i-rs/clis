use crate::models::{Remind, RemindStore};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("reminds.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("reminds.json")
    }
}

pub fn load_store() -> Result<RemindStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(RemindStore::default())
    }
}

pub fn save_store(store: &RemindStore) -> Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn add_remind(store: &mut RemindStore, remind: Remind) {
    store.reminds.insert(remind.name.clone(), remind);
}

pub fn remove_remind(store: &mut RemindStore, name: &str) -> Option<Remind> {
    store.reminds.remove(name)
}

pub fn get_remind<'a>(store: &'a RemindStore, name: &str) -> Option<&'a Remind> {
    store.reminds.get(name)
}

pub fn get_remind_mut<'a>(store: &'a mut RemindStore, name: &str) -> Option<&'a mut Remind> {
    store.reminds.get_mut(name)
}

pub fn filter_by_tag<'a>(store: &'a RemindStore, tag: Option<&str>) -> Vec<&'a Remind> {
    if let Some(tag) = tag {
        store
            .reminds
            .values()
            .filter(|r| r.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.reminds.values().collect()
    }
}
