use crate::models::WalkdogEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(dog_name: String, duration_minutes: i32, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = WalkdogEntry::new(dog_name.clone(), duration_minutes, tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Walked {} for {} minutes", dog_name.green(), duration_minutes.to_string().cyan()));

    Ok(())
}