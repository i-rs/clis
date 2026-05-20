use crate::models::{WaterEntry, WaterStore};
use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

/// List water entries, optionally filtered by tag.
pub fn list_waters(store: &WaterStore, tag: Option<&str>) -> Result<Vec<WaterEntry>> {
    let entries: Vec<WaterEntry> = match tag {
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

/// Get a water entry by id.
pub fn get_water(store: &WaterStore, id: &str) -> Result<WaterEntry> {
    store
        .get_entry(id)
        .cloned()
        .with_context(|| format!("Water record '{id}' not found"))
}

/// Add a new water entry.
pub fn add_water(
    store: &mut WaterStore,
    amount_ml: i32,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<WaterEntry> {
    let now = Utc::now();
    let entry = WaterEntry {
        id: Uuid::new_v4().to_string(),
        amount_ml,
        tags,
        remark,
        drank_at: now,
        created_at: now,
    };
    store.add_entry(entry.clone());
    Ok(entry)
}

/// Update a water entry.
pub fn update_water(
    store: &mut WaterStore,
    id: &str,
    amount_ml: Option<i32>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<WaterEntry> {
    let entry = store
        .get_entry_mut(id)
        .with_context(|| format!("Water record '{id}' not found"))?;

    if let Some(a) = amount_ml {
        entry.amount_ml = a;
    }
    if let Some(t) = tags {
        entry.tags = t;
    }
    if let Some(r) = remark {
        entry.remark = r;
    }

    Ok(entry.clone())
}

/// Delete a water entry by id.
pub fn delete_water(store: &mut WaterStore, id: &str) -> Result<()> {
    store
        .remove_entry(id)
        .ok_or_else(|| anyhow::anyhow!("Water record '{id}' not found"))?;
    Ok(())
}
