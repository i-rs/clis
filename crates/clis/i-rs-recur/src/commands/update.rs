use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::parse_datetime;
use owo_colors::OwoColorize;

pub fn handle_update(
    name: String,
    amount: Option<f64>,
    frequency: Option<String>,
    start_date: Option<String>,
    tag: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = match store.get_entry_mut(&name) {
        Some(e) => e,
        None => {
            anyhow::bail!("Entry '{name}' not found");
        }
    };

    i_rs_core::update_field!(entry.amount, amount);

    i_rs_core::update_field!(entry.frequency, frequency);

    if let Some(start_date) = start_date {
        entry.start_date = parse_datetime(&start_date)?;
    }

    i_rs_core::update_field!(entry.tags, tag);

    i_rs_core::update_field!(entry.remark, remark);

    entry.updated_at = Utc::now();

    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' updated successfully", name.green()));

    Ok(())
}
