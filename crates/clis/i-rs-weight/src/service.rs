use crate::models::{WeightRecord, WeightStore};
use anyhow::{Context, Result};
use chrono::Utc;
use i_rs_core::parse_date;

/// List weight records, optionally filtered by recent days, sorted by date.
pub fn list_weights(store: &WeightStore, days: Option<usize>) -> Result<Vec<WeightRecord>> {
    let mut records: Vec<WeightRecord> = if let Some(d) = days {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(d as i64);
        store
            .entries
            .values()
            .filter(|r| r.date >= cutoff)
            .cloned()
            .collect()
    } else {
        store.entries.values().cloned().collect()
    };
    records.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(records)
}

/// Find a record by short-ID prefix matching.
pub fn find_by_id<'a>(store: &'a WeightStore, id: &str) -> Result<&'a WeightRecord> {
    let record = store
        .entries
        .iter()
        .find(|(key, _)| key.starts_with(id))
        .map(|(_, v)| v)
        .with_context(|| format!("No record found for ID '{id}'"))?;
    Ok(record)
}

/// Get a single weight record by ID.
pub fn get_weight(store: &WeightStore, id: &str) -> Result<WeightRecord> {
    find_by_id(store, id).cloned()
}

/// Add a weight record with UUID. `date_str` defaults to today if None.
pub fn add_weight(
    store: &mut WeightStore,
    date_str: Option<String>,
    weight: f64,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<WeightRecord> {
    let date = match date_str {
        Some(ref s) => parse_date(s)?,
        None => Utc::now().date_naive(),
    };

    let record = WeightRecord {
        id: uuid::Uuid::new_v4().to_string(),
        date,
        weight,
        tags,
        remark,
    };

    store.add_entry(record.clone());
    Ok(record)
}

/// Update a weight record by ID.
pub fn update_weight(
    store: &mut WeightStore,
    id: String,
    weight: Option<f64>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<WeightRecord> {
    let record = find_by_id(store, &id)?;
    let key = record.id.clone();

    let record = store
        .get_entry_mut(&key)
        .with_context(|| format!("No record found for ID '{id}'"))?;

    if let Some(w) = weight {
        record.weight = w;
    }
    if let Some(t) = tags {
        record.tags = t;
    }
    if let Some(r) = remark {
        record.remark = r;
    }

    let updated = record.clone();
    Ok(updated)
}

/// Delete a weight record by ID.
pub fn delete_weight(store: &mut WeightStore, id: String) -> Result<()> {
    let record = find_by_id(store, &id)?;
    let key = record.id.clone();

    if store.remove_entry(&key).is_none() {
        anyhow::bail!("No record found for ID '{id}'");
    }
    Ok(())
}
