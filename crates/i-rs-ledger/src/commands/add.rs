use crate::models::LedgerEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use chrono::{NaiveDate, Utc};
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

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Entry '{}' added successfully", id.green()));
    println!("  {}", format!("ID: {}", &id[..8]).dimmed());

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
