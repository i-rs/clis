use crate::models::KvEntry;
use crate::storage;
use anyhow::{Context, Result};
use chrono::Utc;

/// List KV entries, optionally filtered by tag.
pub fn list_kv(tag: Option<String>) -> Result<Vec<KvEntry>> {
    let store = storage::load_store()?;
    let entries: Vec<KvEntry> = match tag {
        Some(ref t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .cloned()
            .collect(),
        None => store.entries.values().cloned().collect(),
    };
    Ok(entries)
}

/// Get a single KV entry by key.
pub fn get_kv(key: &str) -> Result<KvEntry> {
    let store = storage::load_store()?;
    store
        .get_entry(key)
        .cloned()
        .with_context(|| format!("Key '{key}' not found"))
}

/// Add a new KV entry.
pub fn add_kv(key: String, value: String, tags: Vec<String>, remark: Vec<String>) -> Result<KvEntry> {
    let mut store = storage::load_store()?;

    if store.entries.contains_key(&key) {
        anyhow::bail!("Key '{key}' already exists");
    }

    let now = Utc::now();
    let entry = KvEntry {
        key: key.clone(),
        value,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(entry.clone());
    storage::save_store(&store)?;
    Ok(entry)
}

/// Update a KV entry.
pub fn update_kv(
    key: String,
    value: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<KvEntry> {
    let mut store = storage::load_store()?;

    let entry = store
        .get_entry_mut(&key)
        .with_context(|| format!("Key '{key}' not found"))?;

    if let Some(v) = value {
        entry.value = v;
    }
    if let Some(t) = tags {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }
    entry.updated_at = Utc::now();

    let updated = entry.clone();
    storage::save_store(&store)?;
    Ok(updated)
}

/// Delete a KV entry by key.
pub fn delete_kv(key: &str) -> Result<()> {
    let mut store = storage::load_store()?;

    if store.remove_entry(key).is_none() {
        anyhow::bail!("Key '{key}' not found");
    }

    storage::save_store(&store)?;
    Ok(())
}
