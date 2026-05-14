use crate::models::{TimeEntry, TimeStore};
use anyhow::{Context, Result};
use chrono::Utc;
use dirs::config_dir;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn get_store_path() -> Result<PathBuf> {
    let config_dir = if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir)
    } else {
        config_dir().context("Failed to get config directory")?
    };
    Ok(config_dir.join("i-rs").join("time.json"))
}

pub fn load_store() -> Result<TimeStore> {
    let path = get_store_path()?;

    if !path.exists() {
        return Ok(TimeStore::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))
}

pub fn save_store(store: &TimeStore) -> Result<()> {
    let path = get_store_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let content = serde_json::to_string_pretty(store)
        .context("Failed to serialize store")?;

    fs::write(&path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

pub fn start_timer(store: &mut TimeStore, name: String, tags: Vec<String>, remark: Vec<String>) -> Result<TimeEntry> {
    if let Some(active) = store.get_active_entry() {
        anyhow::bail!("Timer already running: {} (started at {})", active.name, active.start_time.format("%H:%M"));
    }

    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    
    let entry = TimeEntry {
        id: id.clone(),
        name,
        start_time: now,
        end_time: None,
        duration_minutes: 0,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(entry.clone());
    store.active_entry_id = Some(id);

    Ok(store.get_entry(&entry.id).unwrap().clone())
}

pub fn stop_timer(store: &mut TimeStore) -> Result<TimeEntry> {
    let active_id = store.active_entry_id.take()
        .ok_or_else(|| anyhow::anyhow!("No timer is currently running"))?;

    let entry = store.get_entry_mut(&active_id)
        .ok_or_else(|| anyhow::anyhow!("Timer entry not found"))?;

    entry.stop();

    Ok(entry.clone())
}

pub fn delete_entry(store: &mut TimeStore, id: &str) -> Result<TimeEntry> {
    if let Some(active_id) = &store.active_entry_id {
        if active_id == id {
            store.active_entry_id = None;
        }
    }

    store.remove_entry(id)
        .ok_or_else(|| anyhow::anyhow!("Entry '{}' not found", id))
}

pub fn get_entry(store: &TimeStore, id: &str) -> Result<TimeEntry> {
    store.get_entry(id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Entry '{}' not found", id))
}

pub fn list_entries(store: &TimeStore) -> Vec<&TimeEntry> {
    let mut entries: Vec<_> = store.get_all_entries();
    entries.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    entries
}

pub fn list_entries_by_tag<'a>(store: &'a TimeStore, tag: &str) -> Vec<&'a TimeEntry> {
    let tag_str = tag.to_string();
    let mut entries: Vec<&'a TimeEntry> = store
        .get_all_entries()
        .into_iter()
        .filter(|e| e.tags.contains(&tag_str))
        .collect();
    entries.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    entries
}
