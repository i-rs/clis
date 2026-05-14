use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_delete(id: String) -> Result<()> {
    let mut store = storage::load_store()?;

    if storage::remove_quote(&mut store, &id).is_none() {
        anyhow::bail!("Quote '{}' not found", id);
    }

    storage::save_store(&store)?;

    print_success(&format!("✓ Quote '{}' deleted successfully", id.green()));

    Ok(())
}
