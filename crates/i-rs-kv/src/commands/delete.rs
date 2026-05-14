use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(key: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_entry(&mut store, &key).is_none() {
        anyhow::bail!("Key '{}' not found", key);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Key '{}' deleted successfully", key.green()));

    Ok(())
}
