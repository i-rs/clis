use crate::models::AcEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(location: String, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = AcEntry::new(location.clone(), tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded AC cleaning: {}", location.green()));

    Ok(())
}