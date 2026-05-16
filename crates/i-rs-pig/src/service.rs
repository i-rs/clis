use crate::models::{PigEntry, PigStore};
use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

/// List pig entries, optionally filtered by tag.
pub fn list_pigs(store: &PigStore, tag: Option<&str>) -> Result<Vec<PigEntry>> {
    let entries: Vec<PigEntry> = match tag {
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

/// Get a pig entry by id (supports short id matching).
pub fn get_pig(store: &PigStore, id: &str) -> Result<PigEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    store
        .entries
        .values()
        .find(|e| e.id.starts_with(short_id))
        .cloned()
        .with_context(|| format!("Record '{id}' not found"))
}

/// Add a new pig entry.
pub fn add_pig(
    store: &mut PigStore,
    food_name: String,
    description: Option<String>,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<PigEntry> {
    let now = Utc::now();
    let entry = PigEntry {
        id: Uuid::new_v4().to_string(),
        food_name,
        description,
        tags,
        remark,
        happened_at: now,
        created_at: now,
    };

    store.add_entry(entry.clone());
    Ok(entry)
}

/// Update a pig entry.
pub fn update_pig(
    store: &mut PigStore,
    id: &str,
    food_name: Option<String>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<PigEntry> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    let entry = store
        .entries
        .values_mut()
        .find(|e| e.id.starts_with(short_id))
        .with_context(|| format!("Record '{id}' not found"))?;

    if let Some(f) = food_name {
        entry.food_name = f;
    }
    if let Some(d) = description {
        entry.description = d;
    }
    if let Some(t) = tags {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }

    Ok(entry.clone())
}

/// Delete a pig entry by id (supports short id matching).
pub fn delete_pig(store: &mut PigStore, id: &str) -> Result<()> {
    let short_id = if id.len() >= 8 { &id[..8] } else { id };

    if store
        .entries
        .iter()
        .any(|(_, e)| e.id.starts_with(short_id))
    {
        let key = store
            .entries
            .iter()
            .find(|(_, e)| e.id.starts_with(short_id))
            .map(|(k, _)| k.clone())
            .unwrap();
        store.remove_entry(&key);
        Ok(())
    } else {
        anyhow::bail!("Record '{id}' not found")
    }
}
