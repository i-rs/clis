use crate::models::RecurEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::Utc;
use i_rs_core::validate_name;
use owo_colors::OwoColorize;
use i_rs_core::parse_datetime;

pub fn handle_add(
    name: String,
    amount: f64,
    currency: String,
    frequency: String,
    start_date: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    if let Err(e) = validate_name(&name) {
        anyhow::bail!("{}", e.message);
    }

    if store.entries.contains_key(&name) {
        anyhow::bail!("Entry '{}' already exists", name);
    }

    let start = parse_datetime(&start_date)?;

    let now = Utc::now();
    let entry = RecurEntry {
        name: name.clone(),
        amount,
        currency,
        frequency,
        start_date: start,
        tags: tag,
        remark,
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' added successfully", name.green()));

    Ok(())
}


