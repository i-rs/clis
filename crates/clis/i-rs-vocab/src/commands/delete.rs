use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(word_key: String) -> Result<()> {
    let mut store = storage::load_store()?;

    let word_lower = word_key.to_lowercase();
    if store.remove_entry(&word_lower).is_none() {
        anyhow::bail!("Word '{word_key}' not found");
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Word '{}' deleted", word_key.green()));

    Ok(())
}
