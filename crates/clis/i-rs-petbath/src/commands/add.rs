use crate::models::PetbathEntry;
use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_add(pet_name: String, tag: Vec<String>, remark: Vec<String>) -> Result<()> {
    let mut store = storage::load_store()?;

    let entry = PetbathEntry::new(pet_name.clone(), tag, remark);

    store.add_entry(entry);
    storage::save_store(&store)?;

    print_success(&format!("✓ {} took a bath", pet_name.green()));

    Ok(())
}
