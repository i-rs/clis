use crate::models::PurifyEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(filter_type: String, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = PurifyEntry::new(filter_type.clone(), tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Replaced {} filter", filter_type.green()));

    Ok(())
}
