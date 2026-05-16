use crate::models::{SleepRecord, SleepStore};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[allow(dead_code)]
/// List sleep records, optionally filtered by tag.
pub fn list_sleeps(store: &SleepStore, tag: Option<&str>) -> Result<Vec<SleepRecord>> {
    let entries: Vec<SleepRecord> = match tag {
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

#[allow(dead_code)]
/// Get a sleep record by id.
pub fn get_sleep(store: &SleepStore, id: &str) -> Result<SleepRecord> {
    store
        .entries
        .get(id)
        .cloned()
        .with_context(|| format!("Sleep record '{id}' not found"))
}

/// Add a new sleep record.
pub fn add_sleep(
    store: &mut SleepStore,
    bedtime: DateTime<Utc>,
    wake_time: DateTime<Utc>,
    quality: i32,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<SleepRecord> {
    let now = Utc::now();
    let record = SleepRecord {
        id: Uuid::new_v4().to_string(),
        bedtime,
        wake_time,
        quality,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.add_entry(record.clone());
    Ok(record)
}

/// Delete a sleep record by id.
pub fn delete_sleep(store: &mut SleepStore, id: &str) -> Result<SleepRecord> {
    store
        .remove_entry(id)
        .ok_or_else(|| anyhow::anyhow!("Sleep record '{id}' not found"))
}

/// Update a sleep record.
pub fn update_sleep(
    store: &mut SleepStore,
    id: &str,
    bedtime: Option<DateTime<Utc>>,
    wake_time: Option<DateTime<Utc>>,
    quality: Option<i32>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<SleepRecord> {
    let record = store
        .get_entry_mut(id)
        .ok_or_else(|| anyhow::anyhow!("Sleep record '{id}' not found"))?;

    if let Some(b) = bedtime {
        record.bedtime = b;
    }
    if let Some(w) = wake_time {
        record.wake_time = w;
    }
    if let Some(q) = quality {
        record.quality = q;
    }
    if let Some(t) = tags {
        record.tags = t;
    }
    if let Some(r) = remark {
        record.remark = r;
    }
    record.updated_at = Utc::now();

    Ok(record.clone())
}
