use crate::models::{SitEntry, SitStore};
use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

/// List sit entries, optionally filtered by tag.
pub fn list_sits(store: &SitStore, tag: Option<&str>) -> Result<Vec<SitEntry>> {
    let entries: Vec<SitEntry> = match tag {
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

/// Get a sit entry by id (supports short id matching).
pub fn get_sit(store: &SitStore, id: &str) -> Result<SitEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    store
        .entries
        .values()
        .find(|e| e.id.starts_with(short_id))
        .cloned()
        .with_context(|| format!("Record '{id}' not found"))
}

/// Add a new sit entry.
pub fn add_sit(
    store: &mut SitStore,
    duration_minutes: i32,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<SitEntry> {
    let now = Utc::now();
    let started_at = now - chrono::Duration::minutes(i64::from(duration_minutes));
    let entry = SitEntry {
        id: Uuid::new_v4().to_string(),
        duration_minutes,
        started_at,
        ended_at: now,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(entry.clone());
    Ok(entry)
}

/// Update a sit entry.
pub fn update_sit(
    store: &mut SitStore,
    id: &str,
    duration_minutes: Option<i32>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<SitEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let entry = store
        .entries
        .values_mut()
        .find(|e| e.id.starts_with(short_id))
        .with_context(|| format!("Record '{id}' not found"))?;

    if let Some(d) = duration_minutes {
        let now = Utc::now();
        entry.duration_minutes = d;
        entry.started_at = now - chrono::Duration::minutes(i64::from(d));
        entry.ended_at = now;
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

/// Delete a sit entry by id (supports short id matching).
pub fn delete_sit(store: &mut SitStore, id: &str) -> Result<()> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let key = store
        .entries
        .iter()
        .find(|(_, e)| e.id.starts_with(short_id))
        .map(|(k, _)| k.clone())
        .with_context(|| format!("Record '{id}' not found"))?;

    store.remove_entry(&key);
    Ok(())
}
