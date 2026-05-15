use crate::models::WaterEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(amount_ml: i32, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = WaterEntry::new(amount_ml, tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded {}ml at {}", amount_ml.green(), chrono::Utc::now().format("%H:%M").to_string().cyan()));

    Ok(())
}