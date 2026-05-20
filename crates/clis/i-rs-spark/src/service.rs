use crate::models::{SparkEntry, SparkStore};
use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

/// List spark entries, optionally filtered by tag.
pub fn list_sparks(store: &SparkStore, tag: Option<&str>) -> Result<Vec<SparkEntry>> {
    let entries: Vec<SparkEntry> = match tag {
        Some(t) => store
            .entries
            .values()
            .filter(|e| e.tags.iter().any(|tag| tag == t))
            .cloned()
            .collect(),
        None => store.entries.values().cloned().collect(),
    };
    Ok(entries)
}

/// Get a spark entry by id (supports short id matching).
pub fn get_spark(store: &SparkStore, id: &str) -> Result<SparkEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    store
        .entries
        .values()
        .find(|e| e.id.starts_with(short_id))
        .cloned()
        .with_context(|| format!("Spark '{id}' not found"))
}

/// Add a new spark entry.
pub fn add_spark(
    store: &mut SparkStore,
    content: String,
    source: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<SparkEntry> {
    let now = Utc::now();
    let entry = SparkEntry {
        id: Uuid::new_v4().to_string(),
        content,
        source,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(entry.clone());
    Ok(entry)
}

/// Update a spark entry.
pub fn update_spark(
    store: &mut SparkStore,
    id: &str,
    content: Option<String>,
    source: Option<Option<String>>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<SparkEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let entry = store
        .entries
        .values_mut()
        .find(|e| e.id.starts_with(short_id))
        .with_context(|| format!("Spark '{id}' not found"))?;

    if let Some(c) = content {
        entry.content = c;
    }
    if let Some(s) = source {
        entry.source = s;
    }
    if let Some(t) = tags {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }

    entry.updated_at = Utc::now();

    Ok(entry.clone())
}

/// Delete a spark entry by id (supports short id matching).
pub fn delete_spark(store: &mut SparkStore, id: &str) -> Result<()> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let key = store
        .entries
        .iter()
        .find(|(_, e)| e.id.starts_with(short_id))
        .map(|(k, _)| k.clone())
        .with_context(|| format!("Spark '{id}' not found"))?;

    store.remove_entry(&key);
    Ok(())
}
