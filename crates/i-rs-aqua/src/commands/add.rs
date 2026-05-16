use crate::models::AquaEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(tank_size: Option<i32>, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = AquaEntry::new(tank_size, tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    let size_str = tank_size.map_or_else(|| "unknown".to_string(), |s| format!("{s}L"));
    print_success(&format!(
        "✓ Recorded aquarium water change ({})",
        size_str.cyan()
    ));

    Ok(())
}
