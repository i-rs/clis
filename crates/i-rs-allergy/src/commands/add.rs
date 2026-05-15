use crate::models::AllergyEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(
    allergen: String,
    severity: String,
    symptom: Vec<String>,
    tag: Vec<String>,
    remark: Vec<String>,
) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = AllergyEntry::new(allergen.clone(), severity.clone(), symptom, tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ Recorded {} reaction ({} severity)", allergen.green(), severity.yellow()));

    Ok(())
}