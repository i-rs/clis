use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use i_rs_core::parse_date;

pub fn handle_delete(date: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let parsed_date = parse_date(&date)?;

    if store.remove_entry(&parsed_date).is_none() {
        anyhow::bail!("No record for {date}");
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Record for {} deleted", date.green()));

    Ok(())
}


