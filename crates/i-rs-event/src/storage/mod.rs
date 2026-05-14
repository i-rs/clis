use crate::models::{Event, EventStore};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = if let Some(dir) = std::env::var_os("CONFIG_DIR") {
        PathBuf::from(dir)
    } else {
        dirs::config_dir()
            .context("Cannot find config directory")?
            .join("i-rs")
    };

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .context(format!("Failed to create config directory: {:?}", config_dir))?;
    }

    Ok(config_dir)
}

pub fn get_store_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("event.json"))
}

pub fn load_store() -> Result<EventStore> {
    let path = get_store_path()?;

    if !path.exists() {
        return Ok(EventStore::new());
    }

    let content = std::fs::read_to_string(&path)
        .context(format!("Failed to read store from {:?}", path))?;

    serde_json::from_str(&content)
        .context(format!("Failed to parse store from {:?}", path))
}

pub fn save_store(store: &EventStore) -> Result<()> {
    let path = get_store_path()?;
    let content = serde_json::to_string_pretty(store)
        .context("Failed to serialize store")?;

    std::fs::write(&path, content)
        .context(format!("Failed to write store to {:?}", path))?;

    Ok(())
}

pub fn add_event(store: &mut EventStore, event: Event) {
    store.events.insert(event.name.clone(), event);
}

#[allow(dead_code)]
pub fn get_event<'a>(store: &'a EventStore, name: &str) -> Option<&'a Event> {
    store.events.get(name)
}

#[allow(dead_code)]
#[allow(dead_code)]
pub fn get_event_mut<'a>(store: &'a mut EventStore, name: &str) -> Option<&'a mut Event> {
    store.events.get_mut(name)
}

pub fn remove_event(store: &mut EventStore, name: &str) -> bool {
    store.events.remove(name).is_some()
}

#[allow(dead_code)]
pub fn filter_by_tag<'a>(store: &'a EventStore, tag: &str) -> Vec<&'a Event> {
    store
        .events
        .values()
        .filter(|e| e.tags.iter().any(|t| t == tag))
        .collect()
}

#[allow(dead_code)]
pub fn filter_by_type<'a>(store: &'a EventStore, event_type: &str) -> Vec<&'a Event> {
    store
        .events
        .values()
        .filter(|e| e.event_type.to_string() == event_type)
        .collect()
}
