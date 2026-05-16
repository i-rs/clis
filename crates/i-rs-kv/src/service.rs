use crate::models::{KvEntry, KvStats, KvStore};
use anyhow::{Context, Result};
use chrono::Utc;

/// List KV entries, optionally filtered by tag and/or key pattern.
pub fn list_kv(
    store: &KvStore,
    tag: Option<String>,
    pattern: Option<&str>,
) -> Result<Vec<KvEntry>> {
    let entries: Vec<KvEntry> = store
        .entries
        .values()
        .filter(|e| {
            let match_tag = match &tag {
                Some(t) => e.tags.iter().any(|tag| tag == t),
                None => true,
            };
            let match_pattern = match pattern {
                Some(p) => {
                    let pattern_lower = p.to_lowercase();
                    e.key.to_lowercase().contains(&pattern_lower)
                        || e.value.to_lowercase().contains(&pattern_lower)
                }
                None => true,
            };
            match_tag && match_pattern
        })
        .cloned()
        .collect();
    Ok(entries)
}

/// Search entries by value content.
pub fn search_kv(store: &KvStore, query: &str) -> Result<Vec<KvEntry>> {
    let query_lower = query.to_lowercase();
    let entries: Vec<KvEntry> = store
        .entries
        .values()
        .filter(|e| {
            e.key.to_lowercase().contains(&query_lower)
                || e.value.to_lowercase().contains(&query_lower)
                || e.tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
                || e.remark
                    .iter()
                    .any(|r| r.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect();
    Ok(entries)
}

/// Get a single KV entry by key.
pub fn get_kv(store: &KvStore, key: &str) -> Result<KvEntry> {
    store
        .get_entry(key)
        .cloned()
        .with_context(|| format!("Key '{key}' not found"))
}

/// Add a new KV entry.
pub fn add_kv(
    store: &mut KvStore,
    key: String,
    value: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<KvEntry> {
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
    Ok(entry)
}

/// Update a KV entry.
pub fn update_kv(
    store: &mut KvStore,
    key: String,
    value: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<KvEntry> {
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
    Ok(updated)
}

/// Delete a KV entry by key.
pub fn delete_kv(store: &mut KvStore, key: &str) -> Result<()> {
    if store.remove_entry(key).is_none() {
        anyhow::bail!("Key '{key}' not found");
    }

    Ok(())
}

/// Get KV store statistics.
pub fn stats_kv(store: &KvStore) -> Result<KvStats> {
    let entries: Vec<&KvEntry> = store.entries.values().collect();
    let total_entries = entries.len();

    let total_tags: usize = entries.iter().map(|e| e.tags.len()).sum();
    let total_remarks: usize = entries.iter().map(|e| e.remark.len()).sum();
    let total_value_bytes: usize = entries.iter().map(|e| e.value.len()).sum();
    let avg_value_bytes = if total_entries > 0 {
        total_value_bytes as f64 / total_entries as f64
    } else {
        0.0
    };

    let oldest_entry = entries
        .iter()
        .min_by_key(|e| e.created_at)
        .map(|e| e.key.clone());
    let newest_entry = entries
        .iter()
        .max_by_key(|e| e.created_at)
        .map(|e| e.key.clone());

    Ok(KvStats {
        total_entries,
        total_tags,
        total_remarks,
        total_value_bytes,
        avg_value_bytes,
        oldest_entry,
        newest_entry,
    })
}

/// Copy a KV entry to a new key.
pub fn copy_kv(store: &mut KvStore, src: &str, dst: String) -> Result<KvEntry> {
    let original = store
        .get_entry(src)
        .cloned()
        .with_context(|| format!("Key '{src}' not found"))?;

    if store.entries.contains_key(&dst) {
        anyhow::bail!("Key '{dst}' already exists");
    }

    let now = Utc::now();
    let mut copy = original;
    copy.key = dst.clone();
    copy.created_at = now;
    copy.updated_at = now;

    store.add_entry(copy.clone());
    Ok(copy)
}

/// Rename a KV entry key.
pub fn rename_kv(store: &mut KvStore, old: &str, new: String) -> Result<KvEntry> {
    let entry = store
        .get_entry(old)
        .cloned()
        .with_context(|| format!("Key '{old}' not found"))?;

    if store.entries.contains_key(&new) {
        anyhow::bail!("Key '{new}' already exists");
    }

    store.remove_entry(old);

    let now = Utc::now();
    let mut renamed = entry;
    renamed.key = new.clone();
    renamed.updated_at = now;

    store.add_entry(renamed.clone());
    Ok(renamed)
}
