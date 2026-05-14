use crate::models::TowelEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(towel_type: String, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = TowelEntry::new(towel_type.clone(), tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Replaced {} towel", towel_type.green()));

    Ok(())
}