use crate::models::WeightRecord;
use crate::storage;
use anyhow::{Context, Result};
use chrono::Utc;
use i_rs_core::parse_date;

/// List weight records, optionally filtered by recent days.
pub fn list_weights(days: Option<usize>) -> Result<Vec<WeightRecord>> {
    let store = storage::load_store()?;
    let records: Vec<WeightRecord> = if let Some(d) = days {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(d as i64);
        store
            .records
            .values()
            .filter(|r| r.date >= cutoff)
            .cloned()
            .collect()
    } else {
        store.records.values().cloned().collect()
    };
    Ok(records)
}

/// Get a single weight record by date string.
#[allow(dead_code)]
pub fn get_weight(date_str: &str) -> Result<WeightRecord> {
    let date = parse_date(date_str)?;
    let store = storage::load_store()?;
    let record = store
        .records
        .get(&date)
        .cloned()
        .with_context(|| format!("No record found for {date}"))?;
    Ok(record)
}

/// Add a weight record.
pub fn add_weight(date_str: String, weight: f64, remark: Vec<String>) -> Result<WeightRecord> {
    let date = parse_date(&date_str)?;
    let mut store = storage::load_store()?;

    if store.records.contains_key(&date) {
        anyhow::bail!("Record for {date} already exists");
    }

    let record = WeightRecord {
        date,
        weight,
        tags: Vec::new(),
        remark,
    };

    store.add_entry(record.clone());
    storage::save_store(&store)?;
    Ok(record)
}

/// Update a weight record.
pub fn update_weight(date_str: String, weight: Option<f64>, remark: Option<Vec<String>>) -> Result<WeightRecord> {
    let date = parse_date(&date_str)?;
    let mut store = storage::load_store()?;

    let record = store
        .get_entry_mut(&date)
        .with_context(|| format!("No record found for {date}"))?;

    if let Some(w) = weight {
        record.weight = w;
    }
    if let Some(r) = remark {
        record.remark = r;
    }

    let updated = record.clone();
    storage::save_store(&store)?;
    Ok(updated)
}

/// Delete a weight record.
pub fn delete_weight(date_str: String) -> Result<()> {
    let date = parse_date(&date_str)?;
    let mut store = storage::load_store()?;

    if store.remove_entry(&date).is_none() {
        anyhow::bail!("No record found for {date}");
    }

    storage::save_store(&store)?;
    Ok(())
}
