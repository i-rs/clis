use crate::presentation::{print_error, print_success};
use crate::storage;
use anyhow::Result;
use chrono::NaiveDate;
use owo_colors::OwoColorize;

pub fn handle_delete(date: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    if storage::remove_entry(&mut store, &parsed_date).is_none() {
        print_error(&format!("No record for {}", date));
        anyhow::bail!("No record for {}", date);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} deleted", date.green()));

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
