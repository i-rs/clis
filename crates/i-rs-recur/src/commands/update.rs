use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

pub fn handle_update(
    name: String,
    amount: Option<f64>,
    frequency: Option<String>,
    start_date: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match storage::get_entry_mut(&mut store, &name) {
        Some(e) => e,
        None => {
            anyhow::bail!("Entry '{name}' not found");
        }
    };

    if let Some(amount) = amount {
        entry.amount = amount;
    }

    if let Some(frequency) = frequency {
        entry.frequency = frequency;
    }

    if let Some(start_date) = start_date {
        entry.start_date = parse_datetime(&start_date)?;
    }

    if let Some(tag) = tag {
        entry.tags = tag;
    }

    if let Some(remark) = remark {
        entry.remark = remark;
    }

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' updated successfully", name.green()));

    Ok(())
}


