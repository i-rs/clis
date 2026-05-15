use crate::models::LedgerEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use chrono::Utc;
use owo_colors::OwoColorize;
use uuid::Uuid;

pub fn handle_add(
    date: String,
    amount: f64,
    currency: String,
    entry_type: String,
    category: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    let entry = LedgerEntry {
        id: id.clone(),
        date: parsed_date,
        amount,
        currency,
        entry_type,
        category,
        tags: tag,
        remark,
        created_at: now,
    };

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' added successfully", id.green()));
    println!("  {}", format!("ID: {}", &id[..8]).dimmed());

    Ok(())
}

