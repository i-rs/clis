use crate::models::{TimeEntry, TimeStore};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<TimeStore> {
    let mut storage = Storage::<TimeStore>::new("time");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &TimeStore) -> anyhow::Result<()> {
    let storage = Storage::<TimeStore>::new("time");
    storage.save_data(store)
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

    Ok(store.get_entry(&entry.id).expect("entry id was just inserted").clone())
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
