use crate::models::DoseEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    medicine_name: String,
    dosage: String,
    unit: String,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = DoseEntry::new(medicine_name.clone(), dosage.clone(), unit.clone(), tag, remark);

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded {} {} {} at {}", medicine_name.green(), dosage.cyan(), unit.yellow(), chrono::Utc::now().format("%H:%M")));

    Ok(())
}
