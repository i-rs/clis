use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use i_rs_core::parse_date;
use owo_colors::OwoColorize;

pub fn handle_delete(date: String) -> Result<()> {
    let date = parse_date(&date)?;

    let mut store = storage::load_store()?;

    if store.remove_entry(&date).is_none() {
        anyhow::bail!("No record found for {date}");
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} deleted", date.green()));

    Ok(())
}

