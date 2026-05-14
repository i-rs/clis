use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;

pub fn handle_update(
    id: String,
    date: Option<String>,
    amount: Option<f64>,
    entry_type: Option<String>,
    category: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let short_id = if id.len() >= 8 { &id[..8] } else { &id };

    let _entry = match storage::get_entry(&store, short_id) {
        Some(e) => e.clone(),
        None => {
            print_error(&format!("Entry '{}' not found", id));
            anyhow::bail!("Entry '{}' not found", id);
        }
    };

    if let Some(date) = date {
        let parsed_date = parse_date(&date)?;
        let entry_mut = store.entries.get_mut(short_id).unwrap();
        entry_mut.date = parsed_date;
    }

    if let Some(amount) = amount {
        let entry_mut = store.entries.get_mut(short_id).unwrap();
        entry_mut.amount = amount;
    }

    if let Some(entry_type) = entry_type {
        let entry_mut = store.entries.get_mut(short_id).unwrap();
        entry_mut.entry_type = entry_type;
    }

    if let Some(category) = category {
        let entry_mut = store.entries.get_mut(short_id).unwrap();
        entry_mut.category = category;
    }

    if let Some(tag) = tag {
        let entry_mut = store.entries.get_mut(short_id).unwrap();
        entry_mut.tags = tag;
    }

    if let Some(remark) = remark {
        let entry_mut = store.entries.get_mut(short_id).unwrap();
        entry_mut.remark = remark;
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' updated successfully", id.green()));

    Ok(())
}

fn parse_date(date_str: &str) -> Result<NaiveDate> {
    let formats = ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Ok(date);
        }
    }

    Err(anyhow::anyhow!("Invalid date format: {}. Use YYYY-MM-DD", date_str))
}
