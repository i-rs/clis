use crate::models::{KeyEntry, KeyStore};
use crate::storage;
use anyhow::{Context, Result};
use chrono::Utc;

/// List key entries, optionally filtered by tag.
pub fn list_keys(store: &KeyStore, tag: Option<String>) -> Result<Vec<KeyEntry>> {
    let entries: Vec<KeyEntry> = match tag {
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

/// Get a single key entry by name.
pub fn get_key(store: &KeyStore, name: &str) -> Result<KeyEntry> {
    store
        .get_entry(name)
        .cloned()
        .with_context(|| format!("Key '{name}' not found"))
}

/// Add a new key entry. Stores the actual key value in OS keychain.
pub fn add_key(store: &mut KeyStore, name: String, key_type: String, key_value: String, tags: Vec<String>, remark: Vec<String>) -> Result<KeyEntry> {

    if store.entries.contains_key(&name) {
        anyhow::bail!("Key '{name}' already exists");
    }

    // Store value in keychain
    storage::store_key(&name, &key_value)?;

    let now = Utc::now();
    let entry = KeyEntry {
        name: name.clone(),
        key_type,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(entry.clone());
    Ok(entry)
}

/// Update a key entry.
pub fn update_key(
    store: &mut KeyStore,
    name: String,
    key_type: Option<String>,
    key_value: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<KeyEntry> {

    let entry = store
        .get_entry_mut(&name)
        .with_context(|| format!("Key '{name}' not found"))?;

    if let Some(kt) = key_type {
        entry.key_type = kt;
    }
    if let Some(ref kv) = key_value {
        storage::store_key(&name, kv)?;
    }
    if let Some(t) = tags {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }
    entry.updated_at = Utc::now();

    let updated = entry.clone();
    Ok(updated)
}

/// Delete a key entry and its keychain value.
pub fn delete_key(store: &mut KeyStore, name: &str) -> Result<()> {
    if store.remove_entry(name).is_none() {
        anyhow::bail!("Key '{name}' not found");
    }

    storage::delete_key(name)?;
    Ok(())
}